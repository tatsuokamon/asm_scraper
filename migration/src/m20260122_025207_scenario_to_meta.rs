use sea_orm_migration::{prelude::*, sea_orm::ForeignKeyAction};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ScenarioToMeta::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ScenarioToMeta::ScenarioId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ScenarioToMeta::MetaId).text().not_null())
                    .primary_key(
                        Index::create()
                            .col(ScenarioToMeta::ScenarioId)
                            .col(ScenarioToMeta::MetaId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-scenariotometa-scenario")
                            .from(ScenarioToMeta::Table, ScenarioToMeta::ScenarioId)
                            .to(Scenario::Table, Scenario::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-scenariotometa-meta")
                            .from(ScenarioToMeta::Table, ScenarioToMeta::MetaId)
                            .to(Meta::Table, Meta::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ScenarioToMeta::Table).to_owned())
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
pub enum Scenario {
    Table,
    Id,
    Name,
    Url,
}

#[derive(Iden)]
pub enum ScenarioToMeta {
    Table,
    ScenarioId,
    MetaId,
}
