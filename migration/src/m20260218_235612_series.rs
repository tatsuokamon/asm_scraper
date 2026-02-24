use sea_orm_migration::prelude::*; //, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Series::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Series::Id)
                            .integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Series::Works).integer().not_null())
                    .col(ColumnDef::new(Series::Name).text().not_null().unique_key())
                    .col(ColumnDef::new(Series::Url).text().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Series::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Series {
    Table,
    Works,
    Id,
    Name,
    Url,
}
