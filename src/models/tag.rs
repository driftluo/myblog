use crate::{db_wrapper::get_postgres, models::article_tag_relation::Relations};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tags {
    id: Uuid,
    tag: String,
}

impl Tags {
    pub async fn insert(tag: &str) -> Result<u64, String> {
        sqlx::query!(r#"INSERT INTO tags (tag) VALUES ($1)"#, tag)
            .execute(get_postgres())
            .await
            .map(|r| r.rows_affected())
            .map_err(|e| format!("{}", e))
    }

    pub async fn view_list_tag() -> Result<Vec<Tags>, String> {
        sqlx::query_as!(Tags, r#"SELECT * FROM tags"#)
            .fetch_all(get_postgres())
            .await
            .map_err(|e| format!("{}", e))
    }

    pub async fn delete_tag(id: Uuid) -> Result<u64, String> {
        Relations::delete_all(id, false).await;
        sqlx::query!(r#"DELETE FROM tags WHERE id = $1"#, id)
            .execute(get_postgres())
            .await
            .map(|r| r.rows_affected())
            .map_err(|e| format!("{}", e))
    }

    pub async fn edit_tag(&self) -> Result<u64, String> {
        sqlx::query!(
            r#"UPDATE tags SET tag = $1 WHERE id = $2"#,
            self.tag,
            self.id
        )
        .execute(get_postgres())
        .await
        .map(|r| r.rows_affected())
        .map_err(|e| format!("{}", e))
    }
}

#[derive(sqlx::FromRow, Debug, Clone, Deserialize, Serialize)]
pub struct TagCount {
    id: Uuid,
    tag: String,
    count: i64,
}

impl TagCount {
    pub async fn view_tag_count() -> Result<Vec<Self>, String> {
        sqlx::query_as(
            r#"select b.id, b.tag, count(*) from article_tag_relation a join tags b on a.tag_id=b.id group by b.id, b.tag"#
        ).fetch_all(get_postgres())
            .await
            .map_err(|e| format!("{}", e))
    }

    pub async fn view_all_tag_count(limit: i64, offset: i64) -> Result<Vec<Self>, String> {
        sqlx::query_as(
            r#"select a.id, a.tag, (case when b.count is null then 0 else b.count end) as count from tags a left join (select tag_id, count(*) from article_tag_relation group by tag_id) b on a.id = b.tag_id order by a.id limit $1 offset $2"#
        )
            .bind(&limit).bind(&offset).fetch_all(get_postgres())
            .await
            .map_err(|e| format!("{}", e))
    }
}
