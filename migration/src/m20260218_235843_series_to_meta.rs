use sea_orm_migration::{prelude::*, sea_orm::ForeignKeyAction};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SeriesToMeta::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SeriesToMeta::SeriesId).integer().not_null())
                    .col(ColumnDef::new(SeriesToMeta::MetaId).text().not_null())
                    .primary_key(
                        Index::create()
                            .col(SeriesToMeta::SeriesId)
                            .col(SeriesToMeta::MetaId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-seriestometa-series")
                            .from(SeriesToMeta::Table, SeriesToMeta::SeriesId)
                            .to(Series::Table, Series::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-seriestometa-meta")
                            .from(SeriesToMeta::Table, SeriesToMeta::MetaId)
                            .to(Meta::Table, Meta::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SeriesToMeta::Table).to_owned())
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
    Time,
}

#[derive(Iden)]
pub enum Series {
    Table,
    Id,
    Name,
    Url,
}

#[derive(Iden)]
pub enum SeriesToMeta {
    Table,
    SeriesId,
    MetaId,
}
