use sea_orm_migration::{prelude::*};//, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TimeTable::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TimeTable::Id).integer().auto_increment().primary_key())
                    .col(ColumnDef::new(TimeTable::Title).text().not_null())
                    .col(ColumnDef::new(TimeTable::Index).integer().not_null())
                    .col(ColumnDef::new(TimeTable::Time).text().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TimeTable::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum TimeTable {
    Table,
    Id,
    Title,
    Index,
    Time
}
