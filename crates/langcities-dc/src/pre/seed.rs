use std::collections::HashMap;

use langcities_common_server::dto::request::{RequestAccessKind, RequestContext};
use langcities_lcdcdsl::component::Slug;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::dto::vernaculars::CreateVernacularDto;
use crate::entity::{dc_users, vernaculars};
use crate::error::{DcAppError, DcAppErrorTrait};
use crate::repo::user::UserRepo;
use crate::repo::vernacular::VernacularRepo;
use crate::state::AppState;

const SEED_USER_ID: i64 = 1;
const VERNACULAR_COUNT: usize = 10;

#[derive(Clone, Debug)]
pub struct Seeder {
    pub state: AppState,
}

impl Seeder {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn seed_user(&self, auth_user_id: i64) -> Result<dc_users::Model, DcAppError> {
        UserRepo
            .get_or_create_user(&self.state.db, auth_user_id)
            .await
    }

    pub async fn seed_vernaculars(
        &self,
        auth_user_id: i64,
        vernacular_count: usize,
    ) -> Result<HashMap<i64, String>, DcAppError> {
        let slugs = (0..vernacular_count)
            .map(|i| format!("ver{}", i + 1))
            .collect::<Vec<_>>();

        let repo = VernacularRepo::new(self.state.clone());
        for (i, slug) in slugs.iter().enumerate() {
            let dto = CreateVernacularDto {
                slug: slug.parse::<Slug>().unwrap(),
                name: format!("Vernacular {}", i + 1),
            };
            repo.create_vernacular(
                &self.state.db,
                dto,
                auth_user_id,
                RequestContext {
                    caller_id: Some(auth_user_id.into()),
                    access_kind: RequestAccessKind::System,
                },
            )
            .await?;
        }

        let vernacular_records = vernaculars::Entity::find()
            .left_join(dc_users::Entity)
            .filter(dc_users::Column::AuthUserId.eq(auth_user_id))
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
