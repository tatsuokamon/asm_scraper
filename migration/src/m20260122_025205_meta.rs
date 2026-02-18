use sea_orm_migration::{prelude::*}; //, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Meta::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Meta::Id).text().primary_key())
                    .col(ColumnDef::new(Meta::Title).not_null().text())
                    .col(ColumnDef::new(Meta::ImgSrc).text())
                    .col(ColumnDef::new(Meta::Url).text().not_null())
                    .col(ColumnDef::new(Meta::FilePath).text().not_null())
                    .col(ColumnDef::new(Meta::Time).big_integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Meta::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Meta {
    Table,

    Id,
    Title,
    ImgSrc,
    Url,
    FilePath,
    Time
}
