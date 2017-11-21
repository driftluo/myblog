use super::super::users;
use super::super::users::dsl::users as all_users;

use chrono::{ NaiveDateTime, Local };
use diesel;
use diesel::{ FilterDsl, ExpressionMethods, ExecuteDsl, LoadDsl, SelectDsl, FindDsl, PgConnection };
use std::sync::Arc;

use super::super::{ sha3_256_encode, random_string, get_password, RedisPool };

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
            password: reg.password,
            salt,
            nickname: reg.nickname,
            say: reg.say,
            email: reg.email
        }
    }

    pub fn insert(&self, conn: &PgConnection, redis_pool: &Arc<RedisPool>) -> Result<String, String> {
        match diesel::insert(self)
            .into(users::table)
            .get_result::<Users>(conn) {
            Ok(info) => {
                self.set_cookies(redis_pool, info.id)
            }
            Err(err) => {
                Err(format!("{}", err))
            }
        }
    }

    fn set_cookies(&self, redis_pool: &Arc<RedisPool>, id: i32) -> Result<String, String> {
        let cookie = sha3_256_encode(random_string(8));
        let redis_key = "user_".to_string() + &cookie;
        redis_pool.hset(&("user_".to_string() + &cookie), "login_time", Local::now().timestamp());
        redis_pool.hset(&redis_key, "id", id);
        redis_pool.expire(&redis_key, 24 * 3600);
        Ok(cookie)
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
    pub fn view_user_with_cookie(conn: &PgConnection, redis_pool: &Arc<RedisPool>, cookie: &str, admin: &bool) -> Result<Self, String> {
        let redis_key = match admin {
            &true => { "admin_".to_string() + cookie }
            &false => { "user_".to_string() + cookie }
        };
        let id = redis_pool.hget::<i32>(&redis_key, "id");
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
    password: String,
    remember: bool
}

impl LoginUser {
    pub fn verification(&self, conn: &PgConnection, redis_pool: &Arc<RedisPool>, max_age: &Option<i64>) -> Result<String, String> {
        let res = all_users.filter(users::account.eq(self.account.to_owned())).get_result::<Users>(conn);
        match res {
            Ok(data) => {
                if data.password == sha3_256_encode(get_password(&self.password) + &data.salt) {
                    let ttl = match max_age {
                        &Some(t) => t * 3600,
                        &None => 24 * 60
                    };

                    match data.groups {
                        0 => {
                            let cookie = sha3_256_encode(random_string(8));
                            let redis_key = "admin_".to_string() + &cookie;
                            redis_pool.hset(&redis_key, "login_time", Local::now().timestamp());
                            redis_pool.hset(&redis_key, "id", data.id);
                            redis_pool.expire(&redis_key, ttl);
                            Ok(cookie)
                        }
                        _ => {
                            let cookie = sha3_256_encode(random_string(8));
                            let redis_key = "user_".to_string() + &cookie;
                            redis_pool.hset(&("user_".to_string() + &cookie), "login_time", Local::now().timestamp());
                            redis_pool.hset(&redis_key, "id", data.id);
                            redis_pool.expire(&redis_key, ttl);
                            Ok(cookie)
                        }
                    }
                } else {
                    Err(format!("用户或密码错误"))
                }
            }
            Err(err) => {
                Err(format!("{}", err))
            }
        }
    }

    pub fn get_remember(&self) -> bool {
        self.remember
    }

    pub fn sign_out(redis_pool: &Arc<RedisPool>, cookies: &str, admin: &bool) -> bool {
        let redis_key = match admin {
            &true => { "admin_".to_string() + cookies }
            &false => { "user_".to_string() + cookies }
        };

        redis_pool.del(&redis_key)
    }
}
