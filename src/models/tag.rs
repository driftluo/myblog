use super::super::tags;
use super::super::tags::dsl::tags as all_tags;
use super::super::PgConnection;
use super::Relations;

use diesel;
use diesel::{ ExecuteDsl, ExpressionMethods, FilterDsl, LoadDsl };

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct Tags {
    id: i32,
    tag: String
}

impl Tags {
    pub fn view_list_tag(conn: &PgConnection) -> Result<Vec<Tags>, String> {
        let res = all_tags.load::<Tags>(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn delete_tag(conn: &PgConnection, id: i32) -> Result<usize, String> {
        Relations::delete_all(conn, id, "tag");
        let res = diesel::delete(all_tags.filter(tags::id.eq(id)))
        .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn edit_tag(&self, conn: &PgConnection) -> Result<usize, String> {
        let res = diesel::update(all_tags.filter(tags::id.eq(&self.id)))
            .set(tags::tag.eq(&self.tag))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }
}

#[derive(Insertable, Debug, Clone, Deserialize, Serialize)]
#[table_name = "tags"]
pub struct NewTag {
    tag: String
}

impl NewTag {
    pub fn new(tag: &str) -> Self {
        NewTag {
            tag: tag.to_owned()
        }
    }

    pub fn insert(&self, conn: &PgConnection) -> bool {
        diesel::insert(self)
            .into(tags::table)
            .execute(conn)
            .is_ok()
    }

    pub fn insert_with_result(&self, conn: &PgConnection) -> Tags {
        diesel::insert(self)
            .into(tags::table)
            .get_result(conn)
            .unwrap()
    }
}
