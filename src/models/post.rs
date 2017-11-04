use super::super::posts::dsl::posts as all_posts;
use super::super::posts;
use super::super::PgConnection;

use chrono::NaiveDateTime;
use diesel;
use diesel::{ FilterDsl, ExpressionMethods, ExecuteDsl, LoadDsl, SelectDsl, OrderDsl, LimitDsl };

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct Posts {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub published: bool,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
}

impl Posts {
    pub fn delete_with_id(conn: &PgConnection, id: i32) -> Result<usize, String> {
        let res = diesel::delete(all_posts.filter(posts::id.eq(id)))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn query_posts(conn: &PgConnection, id: i32, admin: bool) -> Result<Vec<Posts>, String> {
        let res = if admin {
                all_posts.filter(posts::id.eq(id)).load::<Posts>(conn)
        } else {
                all_posts.filter(posts::id.eq(id)).filter(posts::published.eq(true)).load::<Posts>(conn)
        };

        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn edit_posts(conn: &PgConnection, data: EditArticle) -> Result<usize, String> {
        let res = diesel::update(all_posts.filter(posts::id.eq(data.id)))
            .set((posts::title.eq(data.title), posts::content.eq(data.content)))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn publish_posts(conn: &PgConnection, data: ModifyPublish) -> Result<usize, String> {
        let res = diesel::update(all_posts.filter(posts::id.eq(data.id)))
            .set(posts::published.eq(data.publish))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }
}

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct ArticleList {
    pub id: i32,
    pub title: String,
    pub published: bool,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
}

impl ArticleList {
    pub fn query_list_article(conn: &PgConnection, limit: i64, admin: bool) -> Result<Vec<ArticleList>, String> {
        let res = if admin {
                all_posts
                .select((posts::id, posts::title, posts::published, posts::create_time, posts::modify_time))
                .order(posts::create_time.desc())
                .limit(limit)
                .load::<ArticleList>(conn)
        } else {
            all_posts
                .select((posts::id, posts::title, posts::published, posts::create_time, posts::modify_time))
                .filter(posts::published.eq(true))
                .order(posts::create_time.desc())
                .limit(limit)
                .load::<ArticleList>(conn)
        };

        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }
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

#[derive(Deserialize, Serialize)]
pub struct EditArticle {
    id: i32,
    title: String,
    content: String
}

#[derive(Deserialize, Serialize)]
pub struct ModifyPublish {
    id: i32,
    publish: bool
}
