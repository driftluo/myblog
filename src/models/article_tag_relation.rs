use super::super::article_tag_relation as relation;
use super::super::article_tag_relation::dsl::article_tag_relation as all_relation;
use super::NewTag;

use diesel;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Insertable, Debug, Clone, Deserialize, Serialize)]
#[table_name = "relation"]
pub struct Relations {
    tag_id: Uuid,
    article_id: Uuid,
}

impl Relations {
    pub fn new(article_id: Uuid, tag_id: Uuid) -> Relations {
        Relations { tag_id, article_id }
    }

    pub fn insert(&self, conn: &PgConnection) -> bool {
        diesel::insert_into(relation::table)
            .values(self)
            .execute(conn)
            .is_ok()
    }

    pub fn delete_all(conn: &PgConnection, id: Uuid, method: &str) -> bool {
        if method == "article" {
            diesel::delete(all_relation.filter(relation::article_id.eq(id)))
                .execute(conn)
                .is_ok()
        } else {
            diesel::delete(all_relation.filter(relation::tag_id.eq(id)))
                .execute(conn)
                .is_ok()
        }
    }

    pub fn delete_relation(&self, conn: &PgConnection) -> bool {
        diesel::delete(
            all_relation
                .filter(relation::article_id.eq(self.article_id))
                .filter(relation::tag_id.eq(self.tag_id)),
        ).execute(conn)
            .is_ok()
    }
}

#[derive(Deserialize, Serialize)]
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

    pub fn insert_all(&self, conn: &PgConnection) -> bool {
        // If `tag` exist, insert all the new tags into the table all at once,
        // and return the ID of the newly added tag
        let mut tags_id = if self.tag.is_some() {
            NewTag::insert_all(
                self.tag
                    .clone()
                    .unwrap()
                    .iter()
                    .map(|tag| NewTag::new(tag))
                    .collect::<Vec<NewTag>>(),
                conn,
            )
        } else {
            Vec::new()
        };

        // Combine all tag id
        if self.tag_id.is_some() {
            tags_id.append(&mut self.tag_id.clone().unwrap())
        }

        let new_relations: Vec<Relations> = tags_id
            .iter()
            .map(|id| Relations::new(self.article_id, *id))
            .collect();

        // Insert the relationships into the table
        diesel::insert_into(relation::table)
            .values(&new_relations)
            .execute(conn)
            .is_ok()
    }
}
