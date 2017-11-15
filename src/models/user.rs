use super::super::users;
use super::super::users::dsl::users as all_users;

use super::super::PgConnection;
use chrono::NaiveDateTime;
use diesel;
use diesel::{ FilterDsl, ExpressionMethods, ExecuteDsl, LoadDsl, SelectDsl, FindDsl };

use super::super::{ sha3_256_encode, random_string, get_password };

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct Users {
    pub id: i32,
    pub account: String,
    pub password: String,
    pub salt: String,
    pub groups: i16,
    pub nickname: String,
    pub say: Option<String>,
    pub email: String,
    pub create_time: NaiveDateTime
}

impl Users {
    pub fn delete(conn: &PgConnection, id: i32) -> Result<usize, String> {
        let res = diesel::delete(all_users.find(id))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn edit_user(conn: &PgConnection, data: EditUser) -> Result<usize, String> {
        let res = diesel::update(all_users.filter(users::id.eq(data.id)))
            .set((users::nickname.eq(data.nickname), users::say.eq(data.say), users::email.eq(data.email)))
            .execute(conn);
        match res {
            Ok(num_update) => Ok(num_update),
            Err(err) => Err(format!("{}", err))
        }
    }
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
            password: get_password(&reg.password),
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

impl UserInfo {
    pub fn view_user(conn: &PgConnection, id: i32) -> Result<Self, String> {
        let res = all_users
            .select((users::id, users::account, users::nickname, users::say, users::email, users::create_time))
            .find(id)
            .get_result::<UserInfo>(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }
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
        let password = sha3_256_encode(get_password(&self.new_password) + &salt);
        let res = diesel::update(all_users.filter(users::id.eq(self.id)))
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
                if old.password == sha3_256_encode(get_password(&self.old_password) + &old.salt) {
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
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginUser {
    account: String,
    password: String
}

impl LoginUser {
    pub fn verification(&self, conn: &PgConnection) -> bool {
        let res = all_users.filter(users::account.eq(self.account.to_owned())).get_result::<Users>(conn);
        match res {
            Ok(data) => {
                if data.password == sha3_256_encode(get_password(&self.password) + &data.salt) {
                    match data.groups {
                        0 => {
                            true
                        }
                        _ => {
                            true
                        }
                    }
                } else {
                    false
                }
            }
            Err(err) => {
                println!("{}", err);
                false
            }
        }
    }
}
