use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260929_131653_entry_field_jobs"
    }
}

const EFJ_FK_EF_NAME: &'static str = "fk_entry_field_jobs_entry_field_id_entry_fields";
const EFJA_FK_JOB_NAME: &'static str = "fk_entry_field_job_attempts_job_id_entry_field_jobs";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("entry_field_jobs")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("entry_field_id"))
                    .col(enumeration(
                        "status",
                        "entry_field_job_status",
                        ["QUEUED", "WORKING", "COMPLETED", "FAILED"],
                    ))
                    .col(timestamp_default_now("created_at"))
                    .col(timestamp_default_now("updated_at"))
                    .col(timestamp("expires_at"))
                    .col(integer("n_attempts"))
                    .foreign_key(
                        ForeignKey::create()
                            .name(EFJ_FK_EF_NAME)
                            .from("entry_field_jobs", "entry_field_id")
                            .to("entry_fields", "id")
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("entry_field_job_attempts")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(integer("job_id"))
                    .col(integer("worker_id"))
                    .col(timestamp_default_now("created_at"))
                    .col(timestamp_default_now("updated_at"))
                    .col(timestamp_null("finished_at"))
                    .col(timestamp("expires_at"))
                    .col(text("calculated_value"))
                    .foreign_key(
                        ForeignKey::create()
                            .name(EFJA_FK_JOB_NAME)
                            .from("entry_field_job_attempts", "job_id")
                            .to("entry_field_jobs", "id")
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("entry_field_job_attempts").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("entry_field_jobs").to_owned())
            .await?;

        Ok(())
    }
}
