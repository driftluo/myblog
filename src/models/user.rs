use super::super::users;
use super::super::users::dsl::users as all_users;

use super::super::PgConnection;
use chrono::NaiveDateTime;
use diesel;
use diesel::{ FilterDsl, ExpressionMethods, ExecuteDsl, LoadDsl, SelectDsl, OrderDsl, LimitDsl };

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct Users {
    pub id: i32,
    pub account: String,
    pub password: String,
    pub salt: String,
    pub nickname: String,
    pub say: Option<String>,
    pub email: String,
    pub create_time: NaiveDateTime
}

#[derive(Insertable, Debug, Clone, Deserialize, Serialize)]
#[table_name = "users"]
pub struct NewUser {
    pub account: String,
    pub password: String,
    pub salt: String,
    pub nickname: String,
    pub say: Option<String>,
    pub email: String,
}

impl NewUser {
    pub fn insert(&self, conn: &PgConnection) -> bool {
        diesel::insert(self)
            .into(users::table)
            .execute(conn)
            .is_ok()
    }
}

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub account: String,
    pub nickname: String,
    pub say: Option<String>,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangePassword {
    pub id: i32,
    pub old_password: String,
    pub new_password: String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EditUser {
    pub id: i32,
    pub nickname: String,
    pub say: String,
    pub email: String,
    pub create_time: NaiveDateTime
}
