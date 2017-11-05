use super::super::users;
use super::super::users::dsl::users as all_users;

use super::super::PgConnection;
use chrono::NaiveDateTime;
use diesel;
use diesel::{ FilterDsl, ExpressionMethods, ExecuteDsl, LoadDsl };

use super::super::{ md5_encode, random_string };

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
    pub fn new(reg: RegisteredUser, salt: String) -> Self {
        NewUser {
            account: reg.account,
            password: reg.password,
            salt,
            nickname: reg.nickname,
            say: reg.say,
            email: reg.email
        }
    }

    pub fn insert(&self, conn: &PgConnection) -> bool {
        diesel::insert(self)
            .into(users::table)
            .execute(conn)
            .is_ok()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisteredUser {
    pub account: String,
    pub password: String,
    pub nickname: String,
    pub say: Option<String>,
    pub email: String,
}

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub account: String,
    pub nickname: String,
    pub say: Option<String>,
    pub email: String,
    pub create_time: NaiveDateTime
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangePassword {
    pub id: i32,
    pub old_password: String,
    pub new_password: String
}

impl ChangePassword {
    pub fn change_password(&self, conn: &PgConnection) -> Result<usize, String> {
        let salt = random_string(6);
        let password = md5_encode(self.new_password.to_owned() + &salt);
        let res =  diesel::update(all_users.filter(users::id.eq(self.id)))
            .set((users::password.eq(&password), users::salt.eq(&salt)))
            .execute(conn);
        match res {
            Ok(num_update) => Ok(num_update),
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn verification(&self, conn: &PgConnection) -> bool {
        let old_user = all_users.filter(users::id.eq(self.id)).get_result::<Users>(conn);
        match old_user {
            Ok(old) => {
                if old.password == self.old_password {
                    true
                } else { false }
            }
            Err(_) => false
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EditUser {
    pub id: i32,
    pub nickname: String,
    pub say: String,
    pub email: String,
    pub create_time: NaiveDateTime
}
