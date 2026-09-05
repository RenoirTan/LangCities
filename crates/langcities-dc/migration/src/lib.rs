pub use sea_orm_migration::prelude::*;

mod m20260831_000001_create_users;
mod m20260905_092406_create_vernaculars;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260831_000001_create_users::Migration),
            Box::new(m20260905_092406_create_vernaculars::Migration),
        ]
    }
}
