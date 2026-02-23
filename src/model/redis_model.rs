#[derive(serde::Deserialize)]
pub struct TagSrc {
    pub name: String,
    pub url: String,
}

#[derive(serde::Deserialize)]
pub struct TimeTableSrc {
    pub index: i32,
    pub title: String,
    pub time: String,
}

#[derive(serde::Deserialize)]
pub struct MetaSrc {
    pub id: String,
    pub title: String,
    pub url: String,
    pub img_src: Option<String>,
    pub time: i64,

    pub cv: Vec<TagSrc>,
    pub genre: Vec<TagSrc>,
    pub illust: Vec<TagSrc>,
    pub circle: Vec<TagSrc>,
    pub series: Vec<TagSrc>,
    pub time_table: Vec<TimeTableSrc>,
}

pub type UpdateTagResponse = Vec<TagSrc>;
pub type FindMetaResponse = MetaSrc;
pub type FindDetailResponse = Vec<String>;
// pub type IdxResponse = i32;
