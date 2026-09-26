use sea_orm::{ActiveValue, ConnectionTrait, EntityTrait, TryInsertResult};

use crate::{
    entity::dc_users,
    error::{DcAppError, DcAppErrorTrait},
};

#[derive(Clone, Debug)]
pub struct UserRepo;

impl UserRepo {
    pub async fn get_user_by_id<C>(
        &self,
        conn: &C,
        id: i64,
    ) -> Result<Option<dc_users::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        dc_users::Entity::find_by_id(id)
            .one(conn)
            .await
            .map_err(DcAppError::database)
    }

    pub async fn get_user_by_auth_user_id<C>(
        &self,
        conn: &C,
        auth_user_id: i64,
    ) -> Result<Option<dc_users::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        dc_users::Entity::find_by_auth_user_id(auth_user_id)
            .one(conn)
            .await
            .map_err(DcAppError::database)
    }

    pub async fn create_user<C>(
        &self,
        conn: &C,
        auth_user_id: i64,
    ) -> Result<dc_users::Model, DcAppError>
    where
        C: ConnectionTrait,
    {
        let am = dc_users::ActiveModel {
            auth_user_id: ActiveValue::Set(auth_user_id),
            ..Default::default()
        };
        let result = dc_users::Entity::insert(am)
            .on_conflict_do_nothing_on([dc_users::Column::AuthUserId])
            .exec_with_returning(conn)
            .await
            .map_err(DcAppError::database)?;
        match result {
            TryInsertResult::Empty => Err(DcAppError::database("No SQL executed create_user")),
            TryInsertResult::Conflicted => {
                match self.get_user_by_auth_user_id(conn, auth_user_id).await? {
                    None => Err(DcAppError::database("could not create nor get user")),
                    Some(model) => Ok(model),
                }
            }
            TryInsertResult::Inserted(model) => Ok(model),
        }
    }
}
