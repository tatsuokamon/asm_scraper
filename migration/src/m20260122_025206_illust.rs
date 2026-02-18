use sea_orm_migration::{prelude::*};//, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Illust::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Illust::Id).integer().auto_increment().primary_key())
                    .col(ColumnDef::new(Illust::Works).integer().not_null())
                    .col(ColumnDef::new(Illust::Name).text().not_null().unique_key())
                    .col(ColumnDef::new(Illust::Url).time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Illust::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Illust {
    Table,
    Id,
    Works,
    Name,
    Url
}
