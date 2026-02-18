use sea_orm_migration::{prelude::*};//, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Scenario::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Scenario::Id).integer().auto_increment().primary_key())
                    .col(ColumnDef::new(Scenario::Works).integer().not_null())
                    .col(ColumnDef::new(Scenario::Name).text().not_null().unique_key())
                    .col(ColumnDef::new(Scenario::Url).time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Scenario::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Scenario {
    Table,
    Id,
    Works,
    Name,
    Url
}
