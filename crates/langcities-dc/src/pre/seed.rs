use std::collections::HashMap;

use chrono::Utc;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, Set};

use crate::entity::{dc_users, vernaculars};
use crate::error::{DcAppError, DcAppErrorTrait};
use crate::state::AppState;

const SEED_USER_ID: i64 = 1;
const VERNACULAR_COUNT: usize = 10;

#[derive(Clone, Debug)]
pub struct Seeder<'a> {
    pub state: &'a AppState,
}

impl<'a> Seeder<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn seed_user(&self, user_id: i64) -> Result<(), DcAppError> {
        let user = dc_users::Entity::find_by_id(user_id)
            .one(&self.state.db)
            .await
            .map_err(DcAppError::failed_init)?;

        if user.is_none() {
            dc_users::Entity::insert(dc_users::ActiveModel {
                id: Set(user_id),
                auth_user_id: Set(user_id),
            })
            .exec(&self.state.db)
            .await
            .map_err(DcAppError::failed_init)?;
        }

        Ok(())
    }

    pub async fn seed_vernaculars(
        &self,
        owner_id: i64,
        vernacular_count: usize,
    ) -> Result<HashMap<i64, String>, DcAppError> {
        let slugs = (0..vernacular_count)
            .map(|i| format!("ver{i}"))
            .collect::<Vec<_>>();
        let now = Utc::now();
        let vernaculars = slugs
            .iter()
            .enumerate()
            .map(|(i, slug)| vernaculars::ActiveModel {
                slug: Set(slug.clone()),
                name: Set(format!("Vernacular {i}")),
                updated_at: Set(now),
                created_at: Set(now),
                owner_id: Set(owner_id),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        if !vernaculars.is_empty() {
            match vernaculars::Entity::insert_many(vernaculars)
                .on_conflict(
                    OnConflict::columns([vernaculars::Column::Slug, vernaculars::Column::OwnerId])
                        .do_nothing()
                        .to_owned(),
                )
                .exec(&self.state.db)
                .await
            {
                Ok(_) | Err(DbErr::RecordNotInserted) => {}
                Err(e) => return Err(DcAppError::failed_init(e)),
            }
        }

        let vernacular_records = vernaculars::Entity::find()
            .filter(vernaculars::Column::OwnerId.eq(owner_id))
            .filter(vernaculars::Column::Slug.is_in(slugs))
            .all(&self.state.db)
            .await
            .map_err(DcAppError::failed_init)?;
        let mut vernaculars = HashMap::new();
        for vernacular in vernacular_records {
            vernaculars.insert(vernacular.id, vernacular.slug);
        }
        Ok(vernaculars)
    }

    pub async fn seed_testing(&self) -> Result<(), DcAppError> {
        self.seed_user(SEED_USER_ID).await?;
        self.seed_vernaculars(SEED_USER_ID, VERNACULAR_COUNT)
            .await?;
        Ok(())
    }
}
