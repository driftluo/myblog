use super::super::users;
use super::super::users::dsl::users as all_users;

use chrono::{Local, NaiveDateTime};
use diesel;
use diesel::prelude::*;
use uuid::Uuid;
use serde_json;
use std::sync::Arc;
use super::UserNotify;

use super::super::{get_github_primary_email, get_password, random_string, RedisPool,
                   sha3_256_encode};

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct Users {
    pub id: Uuid,
    pub account: String,
    pub password: String,
    pub salt: String,
    pub groups: i16,
    pub nickname: String,
    pub say: Option<String>,
    pub email: String,
    pub disabled: i16,
    pub create_time: NaiveDateTime,
    pub github: Option<String>,
}

impl Users {
    pub fn delete(
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        id: Uuid,
    ) -> Result<usize, String> {
        let res = diesel::delete(all_users.find(id)).execute(conn);
        match res {
            Ok(data) => {
                UserNotify::remove_with_user(id, redis_pool);
                Ok(data)
            }
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn change_permission(conn: &PgConnection, data: ChangePermission) -> Result<usize, String> {
        let res = diesel::update(all_users.filter(users::id.eq(data.id)))
            .set(users::groups.eq(data.permission))
            .execute(conn);
        match res {
            Ok(num_update) => Ok(num_update),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn into_user_info(self) -> UserInfo {
        UserInfo {
            id: self.id,
            account: self.account,
            nickname: self.nickname,
            groups: self.groups,
            say: self.say,
            email: self.email,
            create_time: self.create_time,
            github: self.github,
        }
    }

    pub fn disabled_user(conn: &PgConnection, data: DisabledUser) -> Result<usize, String> {
        let res = diesel::update(all_users.filter(users::id.eq(data.id)))
            .set(users::disabled.eq(data.disabled))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }
}

#[derive(Insertable, Debug, Clone, Deserialize, Serialize)]
#[table_name = "users"]
struct NewUser {
    pub account: String,
    pub password: String,
    pub salt: String,
    pub nickname: String,
    pub say: Option<String>,
    pub email: String,
    pub github: Option<String>,
}

impl NewUser {
    fn new(reg: RegisteredUser) -> Self {
        let salt = random_string(6);

        NewUser {
            account: reg.account,
            password: sha3_256_encode(get_password(&reg.password) + &salt),
            salt,
            nickname: reg.nickname,
            say: reg.say,
            email: reg.email,
            github: None,
        }
    }

    fn new_with_github(email: String, github: String, account: String, nickname: String) -> Self {
        NewUser {
            account: account,
            password: sha3_256_encode(random_string(8)),
            salt: random_string(6),
            email: email,
            say: None,
            nickname: nickname,
            github: Some(github),
        }
    }

    fn insert(&self, conn: &PgConnection, redis_pool: &Arc<RedisPool>) -> Result<String, String> {
        match diesel::insert_into(users::table)
            .values(self)
            .get_result::<Users>(conn)
        {
            Ok(info) => self.set_cookies(redis_pool, info.into_user_info()),
            Err(err) => Err(format!("{}", err)),
        }
    }

    fn set_cookies(&self, redis_pool: &Arc<RedisPool>, info: UserInfo) -> Result<String, String> {
        let cookie = sha3_256_encode(random_string(8));
        redis_pool.hset(&cookie, "login_time", Local::now().timestamp());
        redis_pool.hset(&cookie, "info", json!(info).to_string());
        redis_pool.expire(&cookie, 24 * 3600);
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

impl RegisteredUser {
    pub fn insert(
        self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
    ) -> Result<String, String> {
        NewUser::new(self).insert(conn, redis_pool)
    }
}

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub account: String,
    pub nickname: String,
    pub groups: i16,
    pub say: Option<String>,
    pub email: String,
    pub create_time: NaiveDateTime,
    pub github: Option<String>,
}

impl UserInfo {
    pub fn view_user(conn: &PgConnection, id: Uuid) -> Result<Self, String> {
        let res = all_users
            .select((
                users::id,
                users::account,
                users::nickname,
                users::groups,
                users::say,
                users::email,
                users::create_time,
                users::github,
            ))
            .filter(users::id.eq(id))
            .get_result::<UserInfo>(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn view_user_with_cookie(redis_pool: &Arc<RedisPool>, cookie: &str) -> String {
        redis_pool.hget::<String>(cookie, "info")
    }

    pub fn view_user_list(
        conn: &PgConnection,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, String> {
        let res = all_users
            .select((
                users::id,
                users::account,
                users::nickname,
                users::groups,
                users::say,
                users::email,
                users::create_time,
                users::github,
            ))
            .limit(limit)
            .offset(offset)
            .order(users::create_time)
            .load::<UserInfo>(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err)),
        }
    }

    /// Get admin information, cache on redis
    /// key is `admin_info`
    pub fn view_admin(conn: &PgConnection, redis_pool: &Arc<RedisPool>) -> Self {
        if redis_pool.exists("admin_info") {
            serde_json::from_str::<UserInfo>(&redis_pool.get("admin_info")).unwrap()
        } else {
            let info = all_users
                .select((
                    users::id,
                    users::account,
                    users::nickname,
                    users::groups,
                    users::say,
                    users::email,
                    users::create_time,
                    users::github,
                ))
                .filter(users::account.eq("admin"))
                .get_result::<UserInfo>(conn)
                .unwrap();
            redis_pool.set("admin_info", &json!(&info).to_string());
            info
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangePassword {
    pub old_password: String,
    pub new_password: String,
}

impl ChangePassword {
    pub fn change_password(
        &self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        cookie: &str,
    ) -> Result<usize, String> {
        let info =
            serde_json::from_str::<UserInfo>(&redis_pool.hget::<String>(cookie, "info")).unwrap();

        if !self.verification(conn, &info.id) {
            return Err("Verification error".to_string());
        }

        let salt = random_string(6);
        let password = sha3_256_encode(get_password(&self.new_password) + &salt);
        let res = diesel::update(all_users.filter(users::id.eq(info.id)))
            .set((users::password.eq(&password), users::salt.eq(&salt)))
            .execute(conn);
        match res {
            Ok(num_update) => Ok(num_update),
            Err(err) => Err(format!("{}", err)),
        }
    }

    fn verification(&self, conn: &PgConnection, id: &Uuid) -> bool {
        let old_user = all_users.filter(users::id.eq(id)).get_result::<Users>(conn);
        match old_user {
            Ok(old) => {
                old.password == sha3_256_encode(get_password(&self.old_password) + &old.salt)
            }
            Err(_) => false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EditUser {
    pub nickname: String,
    pub say: Option<String>,
    pub email: String,
}

impl EditUser {
    pub fn edit_user(
        self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        cookie: &str,
    ) -> Result<usize, String> {
        let info =
            serde_json::from_str::<UserInfo>(&redis_pool.hget::<String>(cookie, "info")).unwrap();
        let res = diesel::update(all_users.filter(users::id.eq(info.id)))
            .set((
                users::nickname.eq(self.nickname),
                users::say.eq(self.say),
                users::email.eq(self.email),
            ))
            .get_result::<Users>(conn);
        match res {
            Ok(data) => {
                redis_pool.hset(cookie, "info", json!(data.into_user_info()).to_string());
                Ok(1)
            }
            Err(err) => Err(format!("{}", err)),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangePermission {
    pub id: Uuid,
    pub permission: i16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginUser {
    account: String,
    password: String,
    remember: bool,
}

impl LoginUser {
    pub fn verification(
        &self,
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        max_age: &Option<i64>,
    ) -> Result<String, String> {
        let res = all_users
            .filter(users::disabled.eq(0))
            .filter(users::account.eq(self.account.to_owned()))
            .get_result::<Users>(conn);
        match res {
            Ok(data) => {
                if data.password == sha3_256_encode(get_password(&self.password) + &data.salt) {
                    let ttl = match *max_age {
                        Some(t) => t * 3600,
                        None => 24 * 60 * 60,
                    };

                    let cookie = sha3_256_encode(random_string(8));
                    redis_pool.hset(&cookie, "login_time", Local::now().timestamp());
                    redis_pool.hset(&cookie, "info", json!(data.into_user_info()).to_string());
                    redis_pool.expire(&cookie, ttl);
                    Ok(cookie)
                } else {
                    Err(String::from("用户或密码错误"))
                }
            }
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn get_remember(&self) -> bool {
        self.remember
    }

    pub fn sign_out(redis_pool: &Arc<RedisPool>, cookies: &str) -> bool {
        redis_pool.del(cookies)
    }

    pub fn login_with_github(
        conn: &PgConnection,
        redis_pool: &Arc<RedisPool>,
        github: String,
        nickname: String,
        account: String,
        token: &str,
    ) -> Result<String, String> {
        let ttl = 24 * 60 * 60;
        match all_users
            .filter(users::disabled.eq(0))
            .filter(users::github.eq(&github))
            .get_result::<Users>(conn)
        {
            // github already exists
            Ok(data) => {
                let cookie = sha3_256_encode(random_string(8));
                redis_pool.hset(&cookie, "login_time", Local::now().timestamp());
                redis_pool.hset(&cookie, "info", json!(data.into_user_info()).to_string());
                redis_pool.expire(&cookie, ttl);
                Ok(cookie)
            }
            Err(_) => {
                let email = match get_github_primary_email(token) {
                    Ok(data) => data,
                    Err(e) => return Err(e),
                };

                match all_users
                    .filter(users::disabled.eq(0))
                    .filter(users::email.eq(&email))
                    .get_result::<Users>(conn)
                {
                    // Account already exists but not linked
                    Ok(data) => {
                        let res = diesel::update(all_users.filter(users::id.eq(data.id)))
                            .set(users::github.eq(github))
                            .get_result::<Users>(conn);
                        match res {
                            Ok(info) => {
                                let cookie = sha3_256_encode(random_string(8));
                                redis_pool.hset(&cookie, "login_time", Local::now().timestamp());
                                redis_pool.hset(
                                    &cookie,
                                    "info",
                                    json!(info.into_user_info()).to_string(),
                                );
                                redis_pool.expire(&cookie, ttl);
                                Ok(cookie)
                            }
                            Err(err) => Err(format!("{}", err)),
                        }
                    }
                    // sign up
                    Err(_) => NewUser::new_with_github(email, github, account, nickname)
                        .insert(conn, redis_pool),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DisabledUser {
    id: Uuid,
    disabled: i16,
}
