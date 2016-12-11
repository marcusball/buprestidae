use chrono::datetime::DateTime;
use chrono::offset::utc::UTC;

#[derive(Queryable)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub is_published: bool,
    pub publish_date: Option<DateTime<UTC>>,
    pub last_modification_date: DateTime<UTC>,
}