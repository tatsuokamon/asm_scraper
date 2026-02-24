use sea_orm_migration::prelude::*; //, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CV::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CV::Id)
                            .integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CV::Works).integer().not_null())
                    .col(ColumnDef::new(CV::Name).text().not_null().unique_key())
                    .col(ColumnDef::new(CV::Url).text().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CV::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum CV {
    Table,
    Works,
    Id,
    Name,
    Url,
}
