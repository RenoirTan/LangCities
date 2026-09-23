use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260923_123303_create_entry_fields"
    }
}

const EFD_PK_NAME: &'static str =
    "pk_entry_field_dependencies_child_entry_field_id_parent_kind_parent_id";

const EFD_FK_CHILD_NAME: &'static str =
    "fk_entry_field_dependencies_child_entry_field_id_entry_fields";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("entry_field_dependencies")
                    .if_not_exists()
                    .col(integer("child_entry_field_id"))
                    .col(small_integer("parent_kind"))
                    .col(integer("parent_id"))
                    .primary_key(
                        Index::create()
                            .name(EFD_PK_NAME)
                            .col("child_entry_field_id")
                            .col("parent_kind")
                            .col("parent_id"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(EFD_FK_CHILD_NAME)
                            .from("entry_field_dependencies", "child_entry_field_id")
                            .to("entry_fields", "id")
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
            .drop_table(Table::drop().table("entry_field_dependencies").to_owned())
            .await?;
        Ok(())
    }
}
