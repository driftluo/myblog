use crate::{
    db_wrapper::get_postgres,
    models::{
        article_tag_relation::{RelationTag, Relations},
        notify::UserNotify,
    },
    utils::markdown_render,
};
use serde::{Deserialize, Serialize};
use sqlx::{
    types::{chrono::NaiveDateTime, Uuid},
    Row,
};
struct InsertArticle<'a> {
    title: &'a str,
    raw_content: &'a str,
    content: String,
}

impl<'a> InsertArticle<'a> {
    fn new(title: &'a str, raw_content: &'a str) -> Self {
        let content = markdown_render(&raw_content);
        InsertArticle {
            title,
            raw_content,
            content,
        }
    }

    async fn insert(self) -> Uuid {
        sqlx::query(
            r#"Insert into articles (title, raw_content, content) VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(self.title)
        .bind(self.raw_content)
        .bind(self.content)
        .map(|row| row.get::<Uuid, _>(0))
        .fetch_one(get_postgres())
        .await
        .unwrap()
    }
}

#[derive(Deserialize, Serialize)]
pub struct NewArticle {
    pub title: String,
    pub raw_content: String,
    pub exist_tags: Option<Vec<Uuid>>,
    pub new_tags: Option<Vec<String>>,
}

impl NewArticle {
    pub async fn insert(self) -> bool {
        let id = InsertArticle::new(&self.title, &self.raw_content)
            .insert()
            .await;
        if self.new_tags.is_some() || self.exist_tags.is_some() {
            RelationTag::new(id, self.new_tags, self.exist_tags)
                .insert_all()
                .await
        } else {
            true
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct EditArticle {
    id: Uuid,
    title: String,
    raw_content: String,
    new_choice_already_exists_tags: Option<Vec<Uuid>>,
    deselect_tags: Option<Vec<Uuid>>,
    new_tags: Option<Vec<String>>,
}

impl EditArticle {
    pub async fn edit_article(self) -> Result<u64, String> {
        let res = sqlx::query!(
            r#"UPDATE articles SET title = $1, content = $2, raw_content = $3 WHERE id = $4"#,
            self.title,
            markdown_render(&self.raw_content),
            self.raw_content,
            self.id
        )
        .execute(get_postgres())
        .await
        .map(|r| r.rows_affected());
        match res {
            Ok(r) => {
                if self.new_tags.is_some() || self.new_choice_already_exists_tags.is_some() {
                    RelationTag::new(self.id, self.new_tags, self.new_choice_already_exists_tags)
                        .insert_all()
                        .await;
                }
                if self.deselect_tags.is_some() {
                    for i in self.deselect_tags.unwrap() {
                        Relations::new(self.id, i).delete_relation().await;
                    }
                }
                Ok(r)
            }
            Err(e) => Err(format!("{}", e)),
        }
    }
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct ArticleList {
    pub id: uuid::Uuid,
    pub title: String,
    pub published: bool,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
}

impl ArticleList {
    pub async fn query_article(
        limit: i64,
        offset: i64,
        admin: bool,
    ) -> Result<Vec<ArticleList>, String> {
        let res = if admin {
            sqlx::query_as!(
                ArticleList,
                r#"SELECT id, title, published, create_time, modify_time
                    FROM articles
                    ORDER BY create_time DESC
                    LIMIT $1 OFFSET $2 "#,
                limit,
                offset
            )
            .fetch_all(get_postgres())
            .await
        } else {
            sqlx::query_as!(
                ArticleList,
                r#"SELECT id, title, published, create_time, modify_time
                    FROM articles
                    WHERE published = true
                    ORDER BY create_time DESC
                    LIMIT $1 OFFSET $2 "#,
                limit,
                offset
            )
            .fetch_all(get_postgres())
            .await
        };

        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub async fn view_unpublished(limit: i64, offset: i64) -> Result<Vec<ArticleList>, String> {
        let res = sqlx::query_as!(
                ArticleList,
                r#"SELECT id, title, published, create_time, modify_time
                    FROM articles
                    WHERE published = false
                    ORDER BY create_time DESC
                    LIMIT $1 OFFSET $2 "#,
                limit,
                offset
            )
            .fetch_all(get_postgres())
            .await;

        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub async fn query_with_tag(tag_id: Uuid) -> Result<Vec<ArticleList>, String> {
        let sql = format!(
            r#"
        SELECT id, title, published, create_time, modify_time FROM article_with_tag
        WHERE ('{}' = any(tags_id)) AND published = true
        ORDER BY create_time DESC"#,
            tag_id
        );
        let res = sqlx::query_as(&sql).fetch_all(get_postgres()).await;
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub async fn size_count() -> usize {
        #[derive(sqlx::FromRow, Default)]
        struct TP {
            count: Option<i64>,
        }
        sqlx::query_as!(TP, r#"select count(*) from articles"#)
            .fetch_one(get_postgres())
            .await
            .unwrap_or_default()
            .count
            .unwrap_or(0) as usize
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArticlesWithTag {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub published: bool,
    pub tags_id: Option<Vec<Uuid>>,
    pub tags: Option<Vec<String>>,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
}

impl ArticlesWithTag {
    pub async fn delete_with_id(id: Uuid) -> Result<u64, String> {
        Relations::delete_all(id, true).await;
        match delete_article(id).await {
            Ok(r) => {
                UserNotify::remove_with_article(id).await;
                Ok(r)
            }
            Err(e) => Err(format!("{}", e)),
        }
    }

    pub async fn query_article(id: Uuid, admin: bool) -> Result<ArticlesWithTag, String> {
        let res = RawArticlesWithTag::query(id, admin).await;
        match res {
            Ok(data) => Ok(data.into_html()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub async fn query_without_article(
        id: Uuid,
        admin: bool,
    ) -> Result<ArticlesWithoutContent, String> {
        let res = RawArticlesWithTag::query(id, admin).await;
        match res {
            Ok(data) => Ok(data.into_without_content()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub async fn query_raw_article(id: Uuid) -> Result<ArticlesWithTag, String> {
        let res = RawArticlesWithTag::query(id, true).await;
        match res {
            Ok(data) => Ok(data.into_markdown()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub async fn publish_article(data: ModifyPublish) -> Result<u64, String> {
        sqlx::query!(
            r#"UPDATE articles SET published = $1 WHERE id = $2"#,
            data.publish,
            data.id
        )
        .execute(get_postgres())
        .await
        .map(|r| r.rows_affected())
        .map_err(|e| format!("{}", e))
    }
}

#[derive(sqlx::FromRow)]
struct RawArticlesWithTag {
    pub id: Uuid,
    pub title: String,
    pub raw_content: String,
    pub content: String,
    pub published: bool,
    pub tags_id: Vec<Option<Uuid>>,
    pub tags: Vec<Option<String>>,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
}

impl RawArticlesWithTag {
    async fn query(uuid: Uuid, admin: bool) -> sqlx::Result<Self> {
        if admin {
            sqlx::query_as(r#"select * from article_with_tag where id = $1"#)
                .bind(&uuid)
                .fetch_one(get_postgres())
                .await
        } else {
            sqlx::query_as(r#"select * from article_with_tag where id = $1 AND published = true"#)
                .bind(&uuid)
                .fetch_one(get_postgres())
                .await
        }
    }

    fn into_markdown(self) -> ArticlesWithTag {
        ArticlesWithTag {
            id: self.id,
            title: self.title,
            content: self.raw_content,
            published: self.published,
            tags_id: {
                Some(
                    self.tags_id
                        .into_iter()
                        .filter_map(|id| if id.is_some() { id } else { None })
                        .collect::<Vec<Uuid>>(),
                )
            },
            tags: {
                Some(
                    self.tags
                        .into_iter()
                        .filter_map(|id| if id.is_some() { id } else { None })
                        .collect::<Vec<String>>(),
                )
            },
            create_time: self.create_time,
            modify_time: self.modify_time,
        }
    }

    fn into_html(self) -> ArticlesWithTag {
        ArticlesWithTag {
            id: self.id,
            title: self.title,
            content: self.content,
            published: self.published,
            tags_id: {
                Some(
                    self.tags_id
                        .into_iter()
                        .filter_map(|id| if id.is_some() { id } else { None })
                        .collect::<Vec<Uuid>>(),
                )
            },
            tags: {
                Some(
                    self.tags
                        .into_iter()
                        .filter_map(|id| if id.is_some() { id } else { None })
                        .collect::<Vec<String>>(),
                )
            },
            create_time: self.create_time,
            modify_time: self.modify_time,
        }
    }

    fn into_without_content(self) -> ArticlesWithoutContent {
        ArticlesWithoutContent {
            id: self.id,
            title: self.title,
            published: self.published,
            tags_id: {
                Some(
                    self.tags_id
                        .into_iter()
                        .filter_map(|id| if id.is_some() { id } else { None })
                        .collect::<Vec<Uuid>>(),
                )
            },
            tags: {
                Some(
                    self.tags
                        .into_iter()
                        .filter_map(|id| if id.is_some() { id } else { None })
                        .collect::<Vec<String>>(),
                )
            },
            create_time: self.create_time,
            modify_time: self.modify_time,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ModifyPublish {
    id: Uuid,
    publish: bool,
}

async fn delete_article(id: Uuid) -> sqlx::Result<u64> {
    sqlx::query!(r#"DELETE FROM articles WHERE id = $1"#, id)
        .execute(get_postgres())
        .await
        .map(|r| r.rows_affected())
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct PublishedStatistics {
    pub dimension: String,
    pub quantity: i64,
}

impl PublishedStatistics {
    pub async fn statistics_published_frequency_by_month(
    ) -> Result<Vec<PublishedStatistics>, String> {
        let res = sqlx::query_as(
            r#"
        SELECT to_char(create_time, 'yyyy-mm') as dimension, count(*) as quantity FROM articles
        GROUP BY dimension ORDER BY dimension;"#,
        )
        .fetch_all(get_postgres())
        .await;
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArticlesWithoutContent {
    pub id: Uuid,
    pub title: String,
    pub published: bool,
    pub tags_id: Option<Vec<Uuid>>,
    pub tags: Option<Vec<String>>,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
}
