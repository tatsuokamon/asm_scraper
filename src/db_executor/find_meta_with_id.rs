use crate::{
    db_executor::{
        db_response::{self, Detail},
        err::DBExecutorErr,
    },
    entity::*,
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, RelationTrait,
};

pub async fn find_meta_with_id(
    meta_id: String,
    db: &DatabaseConnection,
) -> Result<Option<db_response::Detail>, DBExecutorErr> {
    match meta::Entity::find()
        .filter(meta::Column::Id.eq(&meta_id))
        .one(db)
        .await?
    {
        Some(found_meta) => {
            macro_rules! find_sub {
                ($sub_table:ident, $inter_relation:expr, $column:expr, $id:expr) => {
                    $sub_table::Entity::find()
                        .join(sea_orm::JoinType::InnerJoin, $inter_relation.def().rev())
                        .filter($column.eq($id))
                        .all(db)
                        .await?
                        .into_iter()
                        .map(|e| e.into())
                        .collect()
                };
            }
            Ok(Some(Detail {
                cv: find_sub!(
                    cv,
                    cv_to_meta::Relation::Cv,
                    cv_to_meta::Column::MetaId,
                    &meta_id
                ),
                genre: find_sub!(
                    genre,
                    genre_to_meta::Relation::Genre,
                    genre_to_meta::Column::MetaId,
                    &meta_id
                ),
                illust: find_sub!(
                    illust,
                    illust_to_meta::Relation::Illust,
                    illust_to_meta::Column::MetaId,
                    &meta_id
                ),
                circle: find_sub!(
                    circle,
                    circle_to_meta::Relation::Circle,
                    circle_to_meta::Column::MetaId,
                    &meta_id
                ),
                series: find_sub!(
                    series,
                    series_to_meta::Relation::Series,
                    series_to_meta::Column::MetaId,
                    &meta_id
                ),

                time_table: time_table::Entity::find()
                    .filter(time_table::Column::MetaId.eq(&meta_id))
                    .all(db)
                    .await?
                    .into_iter()
                    .map(|e| e.into())
                    .collect(),

                id: meta_id,
                title: found_meta.title,
                url: found_meta.url,
                img_src: found_meta.img_src,
                time: found_meta.time,
                file_path: found_meta.file_path,
            }))
        }
        None => Ok(None),
    }
}
