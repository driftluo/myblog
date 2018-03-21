use super::super::comments;
use super::super::comments::dsl::comments as all_comments;

use chrono::NaiveDateTime;
use uuid::Uuid;
use diesel;
use diesel::prelude::*;
use diesel::sql_types::Text;
use std::sync::Arc;
use serde_json;
use super::super::{RedisPool, UserInfo};

#[derive(Queryable, Debug, Clone, Deserialize, Serialize, QueryableByName)]
#[table_name = "comments"]
pub struct Comments {
    id: Uuid,
    comment: String,
    article_id: Uuid,
    user_id: Uuid,
    #[sql_type = "Text"] nickname: String,
    create_time: NaiveDateTime,
}

impl Comments {
    pub fn query(
        conn: &PgConnection,
        limit: i64,
        offset: i64,
        id: Uuid,
    ) -> Result<Vec<Self>, String> {
        let raw_sql = format!("select a.id, a.comment, a.article_id, a.user_id, b.nickname, a.create_time from comments a join users b on a.user_id=b.id where a.article_id='{}' order by a.create_time limit {} offset {};", id, limit, offset);
        let res = diesel::sql_query(raw_sql).get_results::<Self>(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn delete_with_comment_id(conn: &PgConnection, id: Uuid) -> bool {
        diesel::delete(all_comments.filter(comments::id.eq(id)))
            .execute(conn)
            .is_ok()
    }

    pub fn delete_with_user_id(conn: &PgConnection, id: Uuid) -> bool {
        diesel::delete(all_comments.filter(comments::user_id.eq(id)))
            .execute(conn)
            .is_ok()
    }
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "comments"]
struct InsertComments {
    comment: String,
    article_id: Uuid,
    user_id: Uuid,
}

impl InsertComments {
    fn insert(self, conn: &PgConnection) -> bool {
        diesel::insert_into(comments::table)
            .values(&self)
            .execute(conn)
            .is_ok()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewComments {
    comment: String,
    article_id: Uuid,
}

impl NewComments {
    fn into_insert_comments(self, user_id: Uuid) -> InsertComments {
        InsertComments {
            comment: self.comment,
            article_id: self.article_id,
            user_id,
        }
    }

    pub fn insert(
        self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        cookie: &str,
        admin: &bool,
    ) -> bool {
        let redis_key = match *admin {
            true => "admin_".to_string() + cookie,
            false => "user_".to_string() + cookie,
        };
        let info = serde_json::from_str::<UserInfo>(&redis_pool.hget::<String>(&redis_key, "info"))
            .unwrap();
        self.into_insert_comments(info.id).insert(conn)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteComment {
    comment_id: Uuid,
    user_id: Uuid,
}

impl DeleteComment {
    pub fn delete(
        self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        cookie: &str,
        admin: &bool,
    ) -> bool {
        match *admin {
            true => Comments::delete_with_comment_id(conn, self.comment_id),
            false => {
                let redis_key = "user_".to_string() + cookie;
                let info = serde_json::from_str::<UserInfo>(&redis_pool
                    .hget::<String>(&redis_key, "info"))
                    .unwrap();
                if self.user_id == info.id {
                    Comments::delete_with_comment_id(conn, self.comment_id)
                } else {
                    false
                }
            }
        }
    }
}
