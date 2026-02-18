use sea_orm_migration::{prelude::*, sea_orm::ForeignKeyAction};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TimeTableToMeta::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TimeTableToMeta::TimeTableId).integer().not_null())
                    .col(ColumnDef::new(TimeTableToMeta::MetaId).text().not_null())
                    .primary_key(
                        Index::create().col(TimeTableToMeta::TimeTableId).col(TimeTableToMeta::MetaId)
                    )
                    .foreign_key(
                        ForeignKey::create().name("fk-timetabletometa-timetable")
                        .from(TimeTableToMeta::Table, TimeTableToMeta::TimeTableId)
                        .to(TimeTable::Table, TimeTable::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create().name("fk-timetabletometa-meta")
                        .from(TimeTableToMeta::Table, TimeTableToMeta::MetaId)
                        .to(Meta::Table, Meta::Id).on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TimeTableToMeta::Table).to_owned())
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
pub enum TimeTable {
    Table,
    Id,
    Name,
    Url
}

#[derive(Iden)]
pub enum TimeTableToMeta {
    Table,
    TimeTableId,
    MetaId
}
