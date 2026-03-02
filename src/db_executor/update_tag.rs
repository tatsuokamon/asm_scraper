use crate::{db_executor::TagEntityExt ,model::TagSrc};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection,
    DbErr, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect,
};

#[derive(thiserror::Error, Debug)]
pub enum UpdateTagErr {
    #[error("{0}")]
    DBErr(#[from] sea_orm::DbErr),
}

async fn update_tag<TagRelatedEntity>(tags: &[TagSrc], db: &DatabaseConnection) -> Result<(), DbErr>
where
    TagRelatedEntity: TagEntityExt,
    TagRelatedEntity::Model: IntoActiveModel<TagRelatedEntity::ActiveModel>,
    TagRelatedEntity::ActiveModel: Send,
{
    for tag in tags.iter() {
        if TagRelatedEntity::is_already_exists(&tag.name, db).await? {
            TagRelatedEntity::insert_from_tag(tag, db).await?;
        }
    }

    Ok(())
}
