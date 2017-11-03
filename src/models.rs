use super::posts;
use super::PgConnection;

use chrono::NaiveDateTime;
use diesel;
use diesel::ExecuteDsl;

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct Posts {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub published: bool,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
}

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct ArticleList {
    pub id: i32,
    pub title: String,
    pub published: bool,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
}

#[derive(Insertable, Debug, Clone, Deserialize, Serialize)]
#[table_name = "posts"]
pub struct NewPost {
    pub title: String,
    pub content: String,
}

impl NewPost {
    pub fn new(title: String, content: String) -> Self {
        NewPost {
            title,
            content
        }
    }

    pub fn insert(&self, conn: &PgConnection) -> bool {
        diesel::insert(self)
            .into(posts::table)
            .execute(conn)
            .is_ok()
    }
}
