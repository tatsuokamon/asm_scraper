use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait, TransactionTrait,
};

use crate::{db_executor::db_response, entity::*};

#[derive(thiserror::Error, Debug)]
pub enum FindRoughErr {
    #[error("{0}")]
    DBErr(#[from] sea_orm::DbErr),
}

pub struct TagFinderStruct {
    pub index: Option<i32>,
    pub size: Option<i32>,

    pub cvs: Option<Vec<String>>,
    pub illusts: Option<Vec<String>>,
    pub series: Option<Vec<String>>,
    pub circles: Option<Vec<String>>,
    pub genres: Option<Vec<String>>,
}

pub async fn find_roughs(
    finder: TagFinderStruct,
    db: &DatabaseConnection,
) -> Result<db_response::Roughs, FindRoughErr> {
    let tx = db.begin().await?;

    macro_rules! get_ids_from_names {
        ($names_op:expr, $module:ident) => {{
            if let Some(names) = $names_op {
                let mut ids = Vec::new();
                for item in names {
                    if let Some(model) = $module::Entity::find()
                        .filter($module::Column::Name.eq(&item))
                        .one(&tx)
                        .await?
                    {
                        ids.push(model.id);
                    }
                }
                if !ids.is_empty() { Some(ids) } else { None }
            } else {
                None
            }
        }};
    }

    let cv_ids = get_ids_from_names!(finder.cvs, cv);
    let illust_ids = get_ids_from_names!(finder.illusts, illust);
    let series_ids = get_ids_from_names!(finder.series, series);
    let circle_ids = get_ids_from_names!(finder.circles, circle);
    let genre_ids = get_ids_from_names!(finder.genres, genre);

    let mut db_query = meta::Entity::find();

    macro_rules! query_filter {
        ($query:expr, $ids_op:expr, $inter_relation_def:expr, $inter_filter:expr) => {
            if let Some(ids) = $ids_op {
                $query = $query.join(sea_orm::JoinType::InnerJoin, $inter_relation_def);

                for id in ids {
                    $query = $query.filter($inter_filter.eq(id))
                }
            }
        };
    }

    query_filter!(
        db_query,
        cv_ids,
        cv_to_meta::Relation::Cv.def(),
        cv_to_meta::Column::CvId
    );
    query_filter!(
        db_query,
        illust_ids,
        illust_to_meta::Relation::Illust.def(),
        illust_to_meta::Column::IllustId
    );
    query_filter!(
        db_query,
        series_ids,
        series_to_meta::Relation::Series.def(),
        series_to_meta::Column::SeriesId
    );
    query_filter!(
        db_query,
        circle_ids,
        circle_to_meta::Relation::Circle.def(),
        circle_to_meta::Column::CircleId
    );
    query_filter!(
        db_query,
        genre_ids,
        genre_to_meta::Relation::Genre.def(),
        genre_to_meta::Column::GenreId
    );

    let index = finder.index.unwrap_or(0);
    let size = finder.size.unwrap_or(25);

    Ok(db_response::Roughs {
        index,
        size,
        result: db_query
            .order_by(meta::Column::Time, sea_orm::Order::Asc)
            .offset(Some((index * size).try_into().unwrap()))
            .limit(Some(size.try_into().unwrap()))
            .all(&tx)
            .await?
            .into_iter()
            .map(|e| e.into())
            .collect::<Vec<db_response::Meta>>(),
    })
}
