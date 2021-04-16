use crate::db_wrapper::get_postgres;
use sqlx::Row;
use uuid::Uuid;

pub struct Relations {
    tag_id: Uuid,
    article_id: Uuid,
}

impl Relations {
    pub fn new(article_id: Uuid, tag_id: Uuid) -> Relations {
        Relations { tag_id, article_id }
    }

    async fn insert(&self) -> bool {
        sqlx::query!(
            r#"Insert into article_tag_relation (tag_id, article_id) VALUES ($1, $2)"#,
            self.tag_id,
            self.article_id
        )
        .execute(get_postgres())
        .await
        .is_ok()
    }

    pub async fn delete_all(id: Uuid, filter_by_article: bool) {
        if filter_by_article {
            sqlx::query!(
                r#"DELETE FROM article_tag_relation WHERE article_id = $1"#,
                id
            )
            .execute(get_postgres())
            .await
            .unwrap();
        } else {
            sqlx::query!(r#"DELETE FROM article_tag_relation WHERE tag_id = $1"#, id)
                .execute(get_postgres())
                .await
                .unwrap();
        }
    }

    pub async fn delete_relation(&self) {
        sqlx::query!(
            r#"DELETE FROM article_tag_relation WHERE article_id = $1 AND tag_id = $2"#,
            self.article_id,
            self.tag_id
        )
        .execute(get_postgres())
        .await
        .unwrap();
    }
}

pub struct RelationTag {
    article_id: Uuid,
    tag_id: Option<Vec<Uuid>>,
    tag: Option<Vec<String>>,
}

impl RelationTag {
    pub fn new(article_id: Uuid, tag: Option<Vec<String>>, tag_id: Option<Vec<Uuid>>) -> Self {
        RelationTag {
            article_id,
            tag_id,
            tag,
        }
    }

    pub async fn insert_all(self) -> bool {
        // If `tag` exist, insert all the new tags into the table all at once,
        // and return the ID of the newly added tag
        let mut tags_id = if self.tag.is_some() {
            let tags = self.tag.unwrap();
            // https://github.com/launchbadge/sqlx/issues/294
            sqlx::query(
                r#"INSERT INTO tags (tag)
                SELECT * FROM UNNEST($1)
                RETURNING id"#,
            )
            .bind(&tags)
            .map(|row| row.get::<Uuid, _>(0))
            .fetch_all(get_postgres())
            .await
            .unwrap()
        } else {
            Vec::new()
        };

        // Combine all tag id
        if self.tag_id.is_some() {
            tags_id.append(&mut self.tag_id.unwrap())
        }

        let article_ids: Vec<Uuid> = ::std::iter::repeat(self.article_id)
            .take(tags_id.len())
            .collect();

        // Insert the relationships into the table
        sqlx::query(
            r#"INSERT INTO article_tag_relation (article_id, tag_id)
                SELECT * FROM UNNEST($1, $2)"#,
        )
        .bind(&article_ids)
        .bind(&tags_id)
        .execute(get_postgres())
        .await
        .is_ok()
    }
}
