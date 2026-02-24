use sea_orm_migration::{prelude::*, sea_orm::ForeignKeyAction};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(IllustToMeta::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(IllustToMeta::IllustId).integer().not_null())
                    .col(ColumnDef::new(IllustToMeta::MetaId).text().not_null())
                    .primary_key(
                        Index::create()
                            .col(IllustToMeta::IllustId)
                            .col(IllustToMeta::MetaId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-illusttometa-illust")
                            .from(IllustToMeta::Table, IllustToMeta::IllustId)
                            .to(Illust::Table, Illust::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-illusttometa-meta")
                            .from(IllustToMeta::Table, IllustToMeta::MetaId)
                            .to(Meta::Table, Meta::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(IllustToMeta::Table).to_owned())
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
pub enum Illust {
    Table,
    Id,
    Name,
    Url,
}

#[derive(Iden)]
pub enum IllustToMeta {
    Table,
    IllustId,
    MetaId,
}
