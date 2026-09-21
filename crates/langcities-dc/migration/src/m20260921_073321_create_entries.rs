use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260921_073321_create_entries"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("entries")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("vernacular_id"))
                    .col(integer("index"))
                    .col(timestamp_default_now("created_at"))
                    .index(
                        Index::create()
                            .unique()
                            .name("idx_unique_entries_vernacular_id_index")
                            .col("vernacular_id")
                            .col("index"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_entries_vernacular_id_vernaculars")
                            .from("entries", "vernacular_id")
                            .to("vernaculars", "id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table("entry_fields")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("entry_id"))
                    .col(string("slug"))
                    .col(string("override"))
                    .col(string("dirty_value"))
                    .col(string("clean_value"))
                    .col(timestamp_default_now("updated_at"))
                    .index(
                        Index::create()
                            .unique()
                            .name("idx_unique_entry_fields_entry_id_slug")
                            .col("entry_id")
                            .col("slug"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_entry_fields_entry_id_entries")
                            .from("entry_fields", "entry_id")
                            .to("entries", "id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("entry_fields").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table("entries").to_owned())
            .await?;
        Ok(())
    }
}
