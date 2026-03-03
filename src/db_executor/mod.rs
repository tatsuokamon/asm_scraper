mod create_meta;
mod find_meta_with_id;
mod find_roughs;
mod update_tag;

mod db_response;
mod err;

use crate::{
    db_executor::db_response::{Tag, TimeTable},
    entity::*,
    model::TagSrc,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel,
    QueryFilter, Set,
};

pub use create_meta::create_meta;
pub use db_response::{Detail, Roughs};
pub use err::DBExecutorErr;
pub use find_meta_with_id::find_meta_with_id;
pub use find_roughs::{FindRoughErr, TagFinderStruct, find_roughs};
pub use update_tag::update_tag;

impl From<meta::Model> for db_response::Meta {
    fn from(value: meta::Model) -> Self {
        db_response::Meta {
            id: value.id,
            title: value.title,
            url: value.url,
            img_src: value.img_src,
            time: value.time,
            file_path: value.file_path,
        }
    }
}

macro_rules! impl_into_db_response_tag {
    ($target:ident) => {
        impl From<$target::Model> for Tag {
            fn from(model: $target::Model) -> Self {
                Self {
                    name: model.name,
                    url: model.url,
                }
            }
        }
    };
}

pub trait TagEntityExt: EntityTrait + Send + Sized {
    fn from_tag(tag: &TagSrc) -> Self::ActiveModel;

    async fn find_existing(
        name: &str,
        db: &impl ConnectionTrait,
    ) -> Result<Option<Self::Model>, DbErr>;
    async fn is_already_exists(name: &str, db: &impl ConnectionTrait) -> Result<bool, DbErr>;
    async fn insert_from_tag(tag: &TagSrc, db: &impl ConnectionTrait) -> Result<Self::Model, DbErr>
    where
        <Self as sea_orm::EntityTrait>::Model:
            IntoActiveModel<<Self as sea_orm::EntityTrait>::ActiveModel>,
        <Self as sea_orm::EntityTrait>::ActiveModel: Send,
    {
        Self::from_tag(tag).insert(db).await
    }
}

macro_rules! ImplTagEntityExt {
    ($tag_relate_lib:ident) => {
        impl TagEntityExt for $tag_relate_lib::Entity {
            fn from_tag(tag: &TagSrc) -> Self::ActiveModel {
                Self::ActiveModel {
                    works: Set(0),
                    name: Set(tag.name.clone()),
                    url: Set(tag.url.clone()),
                    ..Default::default()
                }
            }

            async fn find_existing(
                name: &str,
                db: &impl ConnectionTrait,
            ) -> Result<Option<Self::Model>, DbErr> {
                Self::find()
                    .filter($tag_relate_lib::Column::Name.eq(name))
                    .one(db)
                    .await
            }

            async fn is_already_exists(
                name: &str,
                db: &impl ConnectionTrait,
            ) -> Result<bool, DbErr> {
                Ok(Self::find_existing(name, db).await?.is_none())
            }
        }
    };
}

ImplTagEntityExt!(cv);
ImplTagEntityExt!(genre);
ImplTagEntityExt!(illust);
ImplTagEntityExt!(series);
ImplTagEntityExt!(circle);
ImplTagEntityExt!(scenario);

impl_into_db_response_tag!(cv);
impl_into_db_response_tag!(genre);
impl_into_db_response_tag!(circle);
impl_into_db_response_tag!(illust);
impl_into_db_response_tag!(series);
impl_into_db_response_tag!(scenario);

impl From<time_table::Model> for TimeTable {
    fn from(model: time_table::Model) -> Self {
        Self {
            index: model.index,
            title: model.title,
            time: model.time,
        }
    }
}

#[macro_export]
macro_rules! upsert_simple {
    ($meta_id:expr, $srcs:expr, $tx:expr, $sub:ident, $inter:ident, $sub_id:ident, $sub_id_field:ident) => {
        for src in $srcs {
            let existing = if let Some(found) = $sub::Entity::find()
                .filter($sub::Column::Name.eq(&src.name))
                .one($tx)
                .await?
            {
                found
            } else {
                $sub::ActiveModel {
                    name: Set(src.name),
                    url: Set(src.url),
                    works: Set(0),
                    ..Default::default()
                }
                .insert($tx)
                .await?
            };

            let link_exists = $inter::Entity::find()
                .filter($inter::Column::MetaId.eq($meta_id.clone()))
                .filter($inter::Column::$sub_id_field.eq(existing.id))
                .one($tx)
                .await?
                .is_some();
            if !link_exists {
                $inter::Entity::insert($inter::ActiveModel {
                    $sub_id: Set(existing.id),
                    meta_id: Set($meta_id.clone()),
                })
                .on_conflict(
                    OnConflict::columns([$inter::Column::MetaId, $inter::Column::$sub_id_field])
                        .do_nothing()
                        .to_owned(),
                )
                .exec($tx)
                .await?;
                $sub::Entity::update_many()
                    .col_expr($sub::Column::Works, Expr::col($sub::Column::Works).add(1))
                    .filter($sub::Column::Id.eq(existing.id))
                    .exec($tx)
                    .await?;
            }
        }
    };
}
