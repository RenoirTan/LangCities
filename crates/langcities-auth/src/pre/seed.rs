use std::collections::HashMap;

use sea_orm::sea_query::OnConflict;
use sea_orm::sea_query::value::prelude::chrono;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, Set};

use crate::entity::users;
use crate::error::{AuthAppError, AuthAppErrorTrait};
use crate::state::AppState;

#[derive(Clone, Debug)]
pub struct Seeder<'a> {
    pub state: &'a AppState,
}

impl<'a> Seeder<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn seed_many_users(
        &self,
        data: Vec<(String, String)>,
    ) -> Result<HashMap<i64, String>, AuthAppError> {
        let now = chrono::Utc::now();
        let users = data.iter().map(|(u, h)| users::ActiveModel {
            username: Set(u.clone()),
            password_hash: Set(Some(h.clone())),
            created_at: Set(now.clone()),
            updated_at: Set(now),
            ..Default::default()
        });
        match users::Entity::insert_many(users)
            .on_conflict(
                OnConflict::column(users::Column::Username)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&self.state.db)
            .await
        {
            Err(DbErr::RecordNotInserted) => {}
            Err(e) => {
                return Err(AuthAppError::failed_init(Some(e.into())));
            }
            _ => {}
        };
        let user_records = users::Entity::find()
            .filter(
                users::Column::Username
                    .is_in(data.into_iter().map(|(u, _)| u).collect::<Vec<String>>()),
            )
            .all(&self.state.db)
            .await
            .map_err(|e| AuthAppError::failed_init(Some(e.into())))?;
        let mut users = HashMap::new();
        for user in user_records {
            users.insert(user.id, user.username);
        }
        Ok(users)
    }

    pub async fn seed_test_users(
        &self,
        n_users: usize,
    ) -> Result<HashMap<i64, String>, AuthAppError> {
        let mut data = Vec::<(String, String)>::new();
        for i in 0..n_users {
            let username = format!("user{i}");
            let password_hash = self
                .state
                .pw_checker
                .hash_password(format!("password{i}"))
                .await?;
            data.push((username, password_hash));
        }
        self.seed_many_users(data).await
    }

    pub async fn seed_testing(&self) -> Result<(), AuthAppError> {
        self.seed_test_users(10).await?;
        Ok(())
    }
}
