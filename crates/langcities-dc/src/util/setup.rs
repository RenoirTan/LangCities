use crate::entity::{dc_users, prelude::*};
use crate::error::{DcAppError, DcAppErrorTrait};
use sea_orm::ActiveValue::Set;
use sea_orm::{ConnectionTrait, EntityTrait, TransactionTrait, TryInsertResult};

pub async fn create_user_if_not_exists<C: TransactionTrait>(
    conn: &C,
    id: i64,
) -> Result<dc_users::Model, DcAppError> {
    let user = dc_users::ActiveModel {
        auth_user_id: Set(id),
        ..Default::default()
    };
    conn.transaction(|txn| {
        Box::pin(async move {
            match dc_users::Entity::insert(user)
                .on_conflict_do_nothing_on([dc_users::Column::AuthUserId])
                .exec_with_returning(txn)
                .await
            {
                Ok(TryInsertResult::Empty | TryInsertResult::Conflicted) => {
                    dc_users::Entity::find_by_auth_user_id(id).one(txn).await
                }
                Ok(TryInsertResult::Inserted(m)) => Ok(Some(m)),
                Err(e) => Err(e),
            }
        })
    })
    .await
    .map_err(|e| DcAppError::database(Some(e.into())))
    .map(|o| o.ok_or_else(|| DcAppError::database(Some("Could not insert new user".into()))))
    .flatten()
}

pub async fn get_or_create_user<C: ConnectionTrait + TransactionTrait>(
    conn: &C,
    id: i64,
) -> Result<dc_users::Model, DcAppError> {
    match dc_users::Entity::find_by_auth_user_id(id).one(conn).await {
        Ok(Some(u)) => Ok(u),
        Ok(None) => create_user_if_not_exists(conn, id).await,
        Err(e) => Err(DcAppError::database(Some(e.into()))),
    }
}
