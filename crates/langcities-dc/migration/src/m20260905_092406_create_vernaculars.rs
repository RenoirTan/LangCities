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
                    .col(string("slug"))
                    .col(string("name"))
                    .col(timestamp_default_now("updated_at"))
                    .col(timestamp_default_now("created_at"))
                    .col(integer("owner_id"))
                    .col(integer("next_entry_id").default(1))
                    .index(
                        Index::create()
                            .unique()
                            .name("idx_unique_vernaculars_slug_owner_id")
                            .col("slug")
                            .col("owner_id"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_vernaculars_owner_id_dc_users")
                            .from("vernaculars", "owner_id")
                            .to("dc_users", "id")
                            .on_delete(ForeignKeyAction::Cascade)
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
