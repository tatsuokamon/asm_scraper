use sea_orm_migration::{prelude::*};//, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Circle::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Circle::Id).integer().auto_increment().primary_key())
                    .col(ColumnDef::new(Circle::Works).integer().not_null())
                    .col(ColumnDef::new(Circle::Name).text().not_null().unique_key())
                    .col(ColumnDef::new(Circle::Url).time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Circle::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Circle {
    Table,
    Works,
    Id,
    Name,
    Url
}
