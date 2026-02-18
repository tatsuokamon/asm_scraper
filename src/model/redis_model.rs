pub struct TagSrc {
    name: String,
    url: String,
}

pub struct TimeTableSrc {
    index: i32,
    title: String,
    time: String,
}

pub struct MetaSrc {
    id: String,
    title: String,
    url: String,
    img_src: Option<String>,
    time: i64,

    cv: Vec<TagSrc>,
    genre: Vec<TagSrc>,
    illust: Vec<TagSrc>,
    circle: Vec<TagSrc>,
    series: Vec<TagSrc>,
    time_table: Vec<TimeTableSrc>,
}

pub type UpdateTagResponse = Vec<TagSrc>;
pub type FindMetaResponse = MetaSrc;
pub type FindDetailResponse = Vec<String>;
