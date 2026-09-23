pub use sea_orm_migration::prelude::*;

mod m20260831_000001_create_users;
mod m20260905_092406_create_vernaculars;
mod m20260921_073321_create_entries;
mod m20260923_123303_create_entry_field_dependencies;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260831_000001_create_users::Migration),
            Box::new(m20260905_092406_create_vernaculars::Migration),
            Box::new(m20260921_073321_create_entries::Migration),
            Box::new(m20260923_123303_create_entry_field_dependencies::Migration),
        ]
    }
}
