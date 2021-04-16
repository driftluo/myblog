use crate::db_wrapper::get_postgres;
use serde::{Deserialize, Serialize};
use sqlx::types::{chrono::NaiveDateTime, Uuid};

#[derive(sqlx::FromRow, Debug, Clone, Deserialize, Serialize)]
pub struct Comments {
    id: Uuid,
    comment: String,
    article_id: Uuid,
    user_id: Uuid,
    nickname: String,
    create_time: NaiveDateTime,
}

impl Comments {
    pub async fn query(limit: i64, offset: i64, id: Uuid) -> Result<Vec<Self>, String> {
        let sql = format!("SELECT a.id, a.comment, a.article_id, a.user_id, b.nickname, a.create_time FROM comments a JOIN users b ON a.user_id=b.id WHERE a.article_id='{}' ORDER BY a.create_time LIMIT $1 OFFSET $2", id);
        sqlx::query_as(&sql)
            .bind(&limit)
            .bind(&offset)
            .fetch_all(get_postgres())
            .await
            .map_err(|e| format!("{}", e))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewComments {
    comment: String,
    article_id: Uuid,
    reply_user_id: Option<Uuid>,
}

impl NewComments {
    pub async fn insert(&self, user_id: Uuid) -> bool {
        sqlx::query!(
            r#"INSERT INTO comments (comment, article_id, user_id) VALUES ($1, $2, $3)"#,
            self.comment,
            self.article_id,
            user_id
        )
        .execute(get_postgres())
        .await
        .is_ok()
    }

    pub fn reply_user_id(&mut self) -> Option<Uuid> {
        self.reply_user_id.take()
    }

    pub fn article_id(&self) -> Uuid {
        self.article_id
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteComment {
    comment_id: Uuid,
    user_id: Uuid,
}

impl DeleteComment {
    pub async fn delete(self, id: Uuid, permission: Option<i16>) -> bool {
        match permission {
            Some(0) => delete_with_comment_id(self.comment_id).await,
            _ => {
                if self.user_id == id {
                    delete_with_comment_id(self.comment_id).await
                } else {
                    false
                }
            }
        }
    }
}

async fn delete_with_comment_id(comment_id: Uuid) -> bool {
    sqlx::query!(r#"DELETE FROM comments where id = $1"#, comment_id)
        .execute(get_postgres())
        .await
        .is_ok()
}

async fn delete_with_user_id(user_id: Uuid) -> bool {
    sqlx::query!(r#"DELETE FROM comments where user_id = $1"#, user_id)
        .execute(get_postgres())
        .await
        .is_ok()
}
