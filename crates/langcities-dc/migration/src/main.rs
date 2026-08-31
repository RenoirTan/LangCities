use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(langcities_dc_migration::Migrator).await;
}
