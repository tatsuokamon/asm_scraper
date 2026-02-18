use sea_orm_migration::{prelude::*, sea_orm::ForeignKeyAction};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CircleToMeta::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(CircleToMeta::CircleId).integer().not_null())
                    .col(ColumnDef::new(CircleToMeta::MetaId).text().not_null())
                    .primary_key(
                        Index::create().col(CircleToMeta::CircleId).col(CircleToMeta::MetaId)
                    )
                    .foreign_key(
                        ForeignKey::create().name("fk-circletometa-circle")
                        .from(CircleToMeta::Table, CircleToMeta::CircleId)
                        .to(Circle::Table, Circle::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create().name("fk-circletometa-meta")
                        .from(CircleToMeta::Table, CircleToMeta::MetaId)
                        .to(Meta::Table, Meta::Id).on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CircleToMeta::Table).to_owned())
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
pub enum Circle {
    Table,
    Id,
    Name,
    Url
}

#[derive(Iden)]
pub enum CircleToMeta {
    Table,
    CircleId,
    MetaId
}
