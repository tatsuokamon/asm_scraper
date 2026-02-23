#[derive(serde::Serialize)]
pub struct TimeTable {
    pub index: i32,
    pub title: String,
    pub time: String,
}

#[derive(serde::Serialize)]
pub struct Tag {
    pub name: String,
    pub url: String,
}

#[derive(serde::Serialize, Default)]
pub struct Meta {
    pub id: String,
    pub title: String,
    pub url: String,
    pub img_src: Option<String>,
    pub time: i64,
    pub file_path: String,

    pub cv: Vec<Tag>,
    pub genre: Vec<Tag>,
    pub illust: Vec<Tag>,
    pub circle: Vec<Tag>,
    pub series: Vec<Tag>,

    pub time_table: Vec<TimeTable>,
}
