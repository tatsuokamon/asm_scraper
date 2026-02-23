use std::{sync::Arc, time::Duration};

use crate::{
    db_executor::{
        db_response::{self, Meta, Tag, TimeTable},
        err::DBExecutorErr,
    },
    entity::*,
    model::{MetaSrc, TagSrc, UpdateTagResponse},
};
use reqwest::Client;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, RelationTrait,
    TransactionTrait,
    sea_query::{Expr, OnConflict, table},
};

macro_rules! impl_into_db_response_tag {
    ($target:ident) => {
        impl Into<Tag> for $target::Model {
            fn into(self) -> Tag {
                Tag {
                    name: self.name,
                    url: self.url,
                }
            }
        }
    };
}

impl_into_db_response_tag!(cv);
impl_into_db_response_tag!(genre);
impl_into_db_response_tag!(circle);
impl_into_db_response_tag!(illust);
impl_into_db_response_tag!(series);

impl Into<TimeTable> for time_table::Model {
    fn into(self) -> TimeTable {
        TimeTable {
            index: self.index,
            title: self.title,
            time: self.time,
        }
    }
}

macro_rules! create_simple {
    ($tags:expr, $tx:expr) => {
        for tag in $tags {
            let existring = if let Some(found)
        }
    };
}

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
                .on_conflict(OnConflict::columns([
                    $inter::Column::MetaId,
                    $inter::Column::$sub_id_field,
                ]))
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

pub async fn finding_file_path_with_id(
    client: &Arc<Client>,
    id: &String,
) -> Result<String, reqwest::Error> {
    let template_url_file = "https://cdn3.cloudintech.net/file/";
    let template_url_file1 = "https://cdn3.cloudintech.net/file1/";

    let id_for_url = id.to_ascii_uppercase();
    let target = format!("{}{}/{}.m3u8", template_url_file, &id_for_url, &id_for_url);

    Ok(
        match client
            .head(&target)
            .timeout(Duration::from_secs(3))
            .send()
            .await
        {
            // HTTP層以外がErrになるらしいので
            // Timeout以外は単純なreqwest errとしてあつかう
            Err(e) => {
                if e.is_timeout() {
                    Ok(format!(
                        "{}{}/{}.m3u8",
                        template_url_file1, &id_for_url, &id_for_url
                    ))
                } else {
                    Err(e)
                }?
            }
            Ok(res) => match res.error_for_status() {
                Ok(_) => target,
                Err(_) => {
                    format!("{}{}/{}.m3u8", template_url_file1, &id_for_url, &id_for_url)
                }
            },
        },
    )
}

pub async fn find_meta_with_id(
    meta_id: String,
    db: &DatabaseConnection,
) -> Result<Option<db_response::Meta>, DBExecutorErr> {
    match meta::Entity::find()
        .filter(meta::Column::Id.eq(&meta_id))
        .one(db)
        .await?
    {
        Some(found_meta) => {
            let cvs = cv::Entity::find()
                .join(sea_orm::JoinType::InnerJoin, cv_to_meta::Relation::Cv.def())
                .filter(cv_to_meta::Column::MetaId.eq(&meta_id))
                .all(db)
                .await?
                .into_iter()
                .map(|e| e.into())
                .collect();

            let illusts = illust::Entity::find()
                .join(
                    sea_orm::JoinType::InnerJoin,
                    illust_to_meta::Relation::Illust.def(),
                )
                .filter(illust_to_meta::Column::MetaId.eq(&meta_id))
                .all(db)
                .await?
                .into_iter()
                .map(|e| e.into())
                .collect();

            let serieses = series::Entity::find()
                .join(
                    sea_orm::JoinType::InnerJoin,
                    series_to_meta::Relation::Series.def(),
                )
                .filter(series_to_meta::Column::MetaId.eq(&meta_id))
                .all(db)
                .await?
                .into_iter()
                .map(|e| e.into())
                .collect();

            let circles = circle::Entity::find()
                .join(
                    sea_orm::JoinType::InnerJoin,
                    circle_to_meta::Relation::Circle.def(),
                )
                .filter(circle_to_meta::Column::MetaId.eq(&meta_id))
                .all(db)
                .await?
                .into_iter()
                .map(|e| e.into())
                .collect();

            let genres = genre::Entity::find()
                .join(
                    sea_orm::JoinType::InnerJoin,
                    genre_to_meta::Relation::Genre.def(),
                )
                .filter(genre_to_meta::Column::MetaId.eq(&meta_id))
                .all(db)
                .await?
                .into_iter()
                .map(|e| e.into())
                .collect();

            let tables = time_table::Entity::find()
                .filter(time_table::Column::MetaId.eq(&meta_id))
                .all(db)
                .await?
                .into_iter()
                .map(|e| e.into())
                .collect();

            Ok(Some(Meta {
                id: meta_id,
                title: found_meta.title,
                url: found_meta.url,
                img_src: found_meta.img_src,
                time: found_meta.time,
                file_path: found_meta.file_path,

                cv: cvs,
                genre: genres,
                illust: illusts,
                circle: circles,
                series: serieses,

                time_table: tables,
            }))
        }
        None => Ok(None),
    }
}

// pub async fn create_tag(
//     update_tag_resp: UpdateTagResponse,
//     db: &DatabaseConnection,
// ) -> Result<(), DBExecutorErr> {
// }

pub async fn create_meta(
    meta_src: MetaSrc,
    db: &DatabaseConnection,
    client: Arc<Client>,
) -> Result<(), DBExecutorErr> {
    let tx = db.begin().await?;
    let existing_meta_op = meta::Entity::find()
        .filter(meta::Column::Title.eq(&meta_src.title))
        .one(&tx)
        .await?;

    let meta_id = if let Some(existing_meta) = existing_meta_op {
        existing_meta.id
    } else {
        let file_path = finding_file_path_with_id(&client, &meta_src.id).await?;
        let creating_meta = meta::ActiveModel {
            id: Set(meta_src.id),
            file_path: Set(file_path),
            img_src: Set(meta_src.img_src),
            title: Set(meta_src.title),
            url: Set(meta_src.url),
            time: Set(meta_src.time),
        }
        .insert(&tx)
        .await?;

        creating_meta.id
    };
    upsert_simple!(&meta_id, meta_src.cv, &tx, cv, cv_to_meta, cv_id, CvId);
    upsert_simple!(
        &meta_id,
        meta_src.genre,
        &tx,
        genre,
        genre_to_meta,
        genre_id,
        GenreId
    );
    upsert_simple!(
        &meta_id,
        meta_src.circle,
        &tx,
        circle,
        circle_to_meta,
        circle_id,
        CircleId
    );
    upsert_simple!(
        &meta_id,
        meta_src.illust,
        &tx,
        illust,
        illust_to_meta,
        illust_id,
        IllustId
    );
    upsert_simple!(
        &meta_id,
        meta_src.series,
        &tx,
        series,
        series_to_meta,
        series_id,
        SeriesId
    );

    for table in meta_src.time_table {
        if time_table::Entity::find()
            .filter(time_table::Column::Title.eq(&table.title))
            .filter(time_table::Column::Index.eq(table.index))
            .filter(time_table::Column::Time.eq(&table.time))
            .filter(time_table::Column::MetaId.eq(&meta_id))
            .one(&tx)
            .await?
            .is_none()
        {
            time_table::ActiveModel {
                title: Set(table.title),
                index: Set(table.index),
                time: Set(table.time),
                meta_id: Set(meta_id.clone()),
                ..Default::default()
            }
            .insert(&tx)
            .await?;
        };
    }

    Ok(())
}

pub struct DBExecutor {
    db: sea_orm::DatabaseConnection,
    client: reqwest::Client,
}

impl DBExecutor {
    pub async fn insert_meta_src() {}
}
