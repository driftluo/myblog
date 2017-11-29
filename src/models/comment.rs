use super::super::comments;
use super::super::comments::dsl::comments as all_comments;

use chrono::NaiveDateTime;
use uuid::Uuid;
use diesel;
use diesel::{ PgConnection, FilterDsl, ExpressionMethods, LoadDsl, ExecuteDsl, OffsetDsl, OrderDsl, LimitDsl };
use std::sync::Arc;
use serde_json;
use super::super::{ RedisPool, UserInfo };

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct Comments {
    id: Uuid,
    comment: String,
    article_id: Uuid,
    user_id: Uuid,
    user_nickname: String,
    re_user_id: Option<Uuid>,
    re_user_nickname: Option<String>,
    create_time: NaiveDateTime
}

impl Comments {
    pub fn query(conn: &PgConnection, limit: i64, offset: i64, id: Uuid) -> Result<Vec<Self>, String> {
        let res = all_comments.filter(comments::article_id.eq(id))
                    .order(comments::create_time)
                    .limit(limit)
                    .offset(offset)
                    .get_results::<Comments>(conn);
        match res {
            Ok(data) => {
                Ok(data)
            },
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn delete_with_comment_id(conn: &PgConnection, id: Uuid) -> bool {
        diesel::delete(all_comments.filter(comments::id.eq(id)))
            .execute(conn).is_ok()
    }

    pub fn delete_with_user_id(conn: &PgConnection, id: Uuid) -> bool {
        diesel::delete(all_comments.filter(comments::user_id.eq(id)))
            .execute(conn).is_ok()
    }
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "comments"]
struct InsertComments {
    comment: String,
    article_id: Uuid,
    user_id: Uuid,
    user_nickname: String,
    re_user_id: Option<Uuid>,
    re_user_nickname: Option<String>
}

impl InsertComments {
    fn insert(self, conn: &PgConnection) -> bool {
        diesel::insert(&self)
            .into(comments::table)
            .execute(conn).is_ok()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewComments {
    comment: String,
    article_id: Uuid,
    re_user_id: Option<Uuid>,
    re_user_nickname: Option<String>
}

impl NewComments {
    fn into_insert_comments(self, user_id: Uuid, user_nickname: String) -> InsertComments {
        InsertComments {
            comment: self.comment,
            article_id: self.article_id,
            user_id,
            user_nickname,
            re_user_id: self.re_user_id,
            re_user_nickname: self.re_user_nickname
        }
    }

    pub fn insert(self, conn: &PgConnection, redis_pool: &Arc<RedisPool>, cookie: &str, admin: &bool) -> bool {
        let redis_key = match admin {
            &true => { "admin_".to_string() + cookie }
            &false => { "user_".to_string() + cookie }
        };
        let info = serde_json::from_str::<UserInfo>(&redis_pool.hget::<String>(&redis_key, "info")).unwrap();
        self.into_insert_comments(info.id, info.nickname).insert(conn)
    }
}
