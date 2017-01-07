use chrono::datetime::DateTime;
use chrono::offset::utc::UTC;

#[derive(Queryable, Serialize, Debug)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub is_published: bool,
    #[serde(skip_serializing)]
    pub publish_date: Option<DateTime<UTC>>,
    #[serde(skip_serializing)]
    pub last_modification_date: DateTime<UTC>,
}



use super::schema::posts;

#[derive(Insertable)]
#[table_name="posts"]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
    is_published: bool,
    publish_date: Option<DateTime<UTC>>,
    last_modification_date: DateTime<UTC>,
}

impl<'a> NewPost<'a> {
    pub fn draft(title: &'a str, body: &'a str) -> NewPost<'a> {
        NewPost {
            title: title,
            body: body,
            is_published: false,
            publish_date: None,
            last_modification_date: UTC::now(),
        }
    }

    pub fn new(title: &'a str, body: &'a str) -> NewPost<'a> {
        NewPost {
            title: title,
            body: body,
            is_published: true,
            publish_date: Some(UTC::now()),
            last_modification_date: UTC::now(),
        }
    }
}

#[derive(Queryable)]
pub struct User {
    /// Unique ID representing this user
    pub id: i32,

    /// The full name for this user. Eg: "Jane Doe".
    pub name: String,

    /// The display name for this user. Eg: "Jane".
    pub display_name: String,

    /// The email address for this user.
    pub email: String,

    /// The secret, base32 code, which is used to generate OTP codes for logging in.
    pub code: String,
}