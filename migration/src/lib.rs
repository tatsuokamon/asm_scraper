pub use sea_orm_migration::prelude::*;

mod m20260122_025205_meta;
mod m20260122_025206_cv;
mod m20260122_025206_illust;
mod m20260122_025206_scenario;
mod m20260122_025206_time_table;
mod m20260122_025207_circle;
mod m20260122_025207_cv_to_meta;
mod m20260122_025207_genre;
mod m20260122_025207_scenario_to_meta;
mod m20260122_025208_circle_to_meta;
mod m20260122_025208_genre_to_meta;
mod m20260122_025208_illust_to_meta;
mod m20260218_235612_series;
mod m20260218_235843_series_to_meta;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260122_025205_meta::Migration),
            Box::new(m20260122_025206_time_table::Migration),
            Box::new(m20260122_025206_cv::Migration),
            Box::new(m20260122_025206_scenario::Migration),
            Box::new(m20260122_025206_illust::Migration),
            Box::new(m20260122_025207_genre::Migration),
            Box::new(m20260122_025207_circle::Migration),
            Box::new(m20260122_025207_cv_to_meta::Migration),
            Box::new(m20260122_025207_scenario_to_meta::Migration),
            Box::new(m20260122_025208_circle_to_meta::Migration),
            Box::new(m20260122_025208_illust_to_meta::Migration),
            Box::new(m20260122_025208_genre_to_meta::Migration),
            Box::new(m20260218_235612_series::Migration),
            Box::new(m20260218_235843_series_to_meta::Migration),
        ]
    }
}
