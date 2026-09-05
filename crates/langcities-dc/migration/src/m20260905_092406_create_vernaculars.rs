use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260905_092406_create_vernaculars"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table("vernaculars")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(string("slug").unique_key())
                    .col(string("name"))
                    .col(timestamp("updated_at"))
                    .col(timestamp("created_at"))
                    .col(integer_null("owner_id"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_vernaculars_owner_id_dc_users")
                            .from("vernaculars", "owner_id")
                            .to("dc_users", "id")
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table("vernaculars").to_owned())
            .await
    }
}
