use sea_orm_migration::{prelude::*, sea_orm::ForeignKeyAction};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GenreToMeta::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GenreToMeta::GenreId).integer().not_null())
                    .col(ColumnDef::new(GenreToMeta::MetaId).text().not_null())
                    .primary_key(
                        Index::create().col(GenreToMeta::GenreId).col(GenreToMeta::MetaId)
                    )
                    .foreign_key(
                        ForeignKey::create().name("fk-genretometa-genre")
                        .from(GenreToMeta::Table, GenreToMeta::GenreId)
                        .to(Genre::Table, Genre::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create().name("fk-genretometa-meta")
                        .from(GenreToMeta::Table, GenreToMeta::MetaId)
                        .to(Meta::Table, Meta::Id).on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GenreToMeta::Table).to_owned())
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
    Time
}

#[derive(Iden)]
pub enum Genre {
    Table,
    Id,
    Name,
    Url
}

#[derive(Iden)]
pub enum GenreToMeta {
    Table,
    GenreId,
    MetaId
}
