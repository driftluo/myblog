use super::super::comments;
use super::super::comments::dsl::comments as all_comments;

use uuid::Uuid;
use diesel::{ PgConnection, FilterDsl, ExpressionMethods, LoadDsl };

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
struct Comments {
    id: Uuid,
    comment: String,
    article_id: Uuid,
    user_id: Uuid,
    re_user_id: Option<Uuid>
}

impl Comments {
    pub fn query(conn: &PgConnection, id: Uuid) -> Result<Self, String> {
        let res = all_comments.filter(comments::id.eq(id)).get_result::<Comments>(conn);
        match res {
            Ok(data) => {
                Ok(data)
            },
            Err(err) => Err(format!("{}", err))
        }
    }
}
