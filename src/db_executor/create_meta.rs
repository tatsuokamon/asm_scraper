use std::{sync::Arc, time::Duration};

use crate::{db_executor::err::DBExecutorErr, entity::*, model::MetaSrc, upsert_simple};
use reqwest::Client;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
    sea_query::{Expr, OnConflict},
};

async fn finding_file_path_with_id(
    client: &Arc<Client>,
    id: &str,
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
    tx.commit().await?;

    Ok(())
}
