use super::super::article_tag_relation as relation;
use super::super::article_tag_relation::dsl::article_tag_relation as all_relation;
use super::NewTag;

use diesel;
use diesel::{ ExecuteDsl, ExpressionMethods, FilterDsl, PgConnection };

#[derive(Insertable, Debug, Clone, Deserialize, Serialize)]
#[table_name="relation"]
pub struct Relations {
    tag_id: i32,
    article_id: i32
}

impl Relations {
    fn new(article_id: i32, tag_id: i32) -> Relations {
        Relations {
            tag_id,
            article_id
        }
    }

    pub fn insert(&self, conn: &PgConnection) -> bool {
        diesel::insert(self)
            .into(relation::table)
            .execute(conn)
            .is_ok()
    }

    pub fn delete_all(conn: &PgConnection, id: i32, method: &str) -> bool {
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
        diesel::delete(all_relation.filter(relation::article_id.eq(self.article_id))
            .filter(relation::tag_id.eq(self.tag_id)))
            .execute(conn)
            .is_ok()
    }
}

#[derive(Deserialize, Serialize)]
pub struct RelationTag {
    article_id: i32,
    tag_id: Option<i32>,
    tag: String,
}

impl RelationTag {
    pub fn insert(&self, conn: &PgConnection) -> bool {
        match self.tag_id {
            Some(id) => {
                Relations::new(self.article_id, id).insert(conn)
            }
            None => {
                let tag = NewTag::new(&self.tag).insert_with_result(conn);
                Relations::new(self.article_id, tag.get_id()).insert(conn)
            }
        }
    }
}
