use crate::{
    db_wrapper::{get_postgres, get_redis},
    models::notify::UserNotify,
    utils::{
        get_password, github_information::get_github_primary_email, random_string, sha3_256_encode,
    },
};
use serde::{Deserialize, Serialize};
use sqlx::types::{
    chrono::{Local, NaiveDateTime},
    Uuid,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    pub async fn delete(id: Uuid) -> Result<u64, String> {
        let res = sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, id)
            .execute(get_postgres())
            .await
            .map(|r| r.rows_affected())
            .map_err(|e| format!("{}", e))?;
        UserNotify::remove_with_user(id).await;
        Ok(res)
    }

    pub async fn change_permission(data: ChangePermission) -> Result<u64, String> {
        sqlx::query!(
            r#"UPDATE users SET groups = $1 WHERE id = $2"#,
            data.permission,
            data.id
        )
        .execute(get_postgres())
        .await
        .map(|r| r.rows_affected())
        .map_err(|e| format!("{}", e))
    }

    pub async fn disabled_user(data: DisabledUser) -> Result<u64, String> {
        sqlx::query!(
            r#"UPDATE users SET disabled = $1 WHERE id = $2"#,
            data.disabled,
            data.id
        )
        .execute(get_postgres())
        .await
        .map(|r| r.rows_affected())
        .map_err(|e| format!("{}", e))
    }

    pub async fn view_user(id: Uuid) -> Result<Self, String> {
        sqlx::query_as!(
            UserInfo,
            r#"SELECT id, account, nickname, groups, say, email, create_time, github FROM users
            WHERE id = $1"#,
            id
        )
        .fetch_one(get_postgres())
        .await
        .map_err(|e| format!("{}", e))
    }

    pub async fn view_user_with_github(github: &str) -> Result<Self, String> {
        sqlx::query_as!(
            UserInfo,
            r#"SELECT id, account, nickname, groups, say, email, create_time, github FROM users
            WHERE github = $1 AND disabled = 0"#,
            github
        )
        .fetch_one(get_postgres())
        .await
        .map_err(|e| format!("{}", e))
    }

    pub async fn view_user_with_email(email: &str) -> Result<Self, String> {
        sqlx::query_as!(
            UserInfo,
            r#"SELECT id, account, nickname, groups, say, email, create_time, github FROM users
            WHERE email = $1 AND disabled = 0"#,
            email
        )
        .fetch_one(get_postgres())
        .await
        .map_err(|e| format!("{}", e))
    }

    pub async fn view_user_with_cookie(cookie: &str) -> String {
        get_redis().hget::<String>(cookie, "info").await.unwrap()
    }

    pub async fn view_user_list(limit: i64, offset: i64) -> Result<Vec<Self>, String> {
        sqlx::query_as!(
            UserInfo,
            r#"SELECT id, account, nickname, groups, say, email, create_time, github FROM users
            ORDER BY create_time
            LIMIT $1 OFFSET $2"#,
            limit,
            offset
        )
        .fetch_all(get_postgres())
        .await
        .map_err(|e| format!("{}", e))
    }

    /// Get admin information, cache on redis
    /// key is `admin_info`
    pub async fn view_admin() -> Self {
        let redis_pool = get_redis();
        if let Ok(Some(info)) = redis_pool.get("admin_info").await {
            serde_json::from_str::<UserInfo>(&info).unwrap()
        } else {
            let info = sqlx::query_as!(
                UserInfo,
                r#"SELECT id, account, nickname, groups, say, email, create_time, github FROM users WHERE account = 'admin'"#,
            )
                .fetch_one(get_postgres())
                .await
                .unwrap();
            redis_pool
                .set("admin_info", &serde_json::json!(&info).to_string())
                .await;
            info
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    pub fn new(reg: RegisteredUser) -> Self {
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
    pub fn new_with_github(
        email: String,
        github: String,
        account: String,
        nickname: String,
    ) -> Self {
        NewUser {
            account,
            password: sha3_256_encode(random_string(8)),
            salt: random_string(6),
            email,
            say: None,
            nickname,
            github: Some(github),
        }
    }

    pub async fn insert(self) -> Result<String, String> {
        let res = sqlx::query_as!(
            UserInfo,
            r#"INSERT INTO users (account, password, salt, nickname, say, email, github)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, account, nickname, groups, say, email, create_time, github"#,
            self.account,
            self.password,
            self.salt,
            self.nickname,
            self.say,
            self.email,
            self.github
        )
        .fetch_one(get_postgres())
        .await;
        match res {
            Ok(info) => self.set_cookies(info).await,
            Err(err) => Err(format!("{}", err)),
        }
    }

    async fn set_cookies(&self, info: UserInfo) -> Result<String, String> {
        let cookie = sha3_256_encode(random_string(8));
        let redis_pool = get_redis();
        redis_pool
            .hset(&cookie, "login_time", Local::now().timestamp())
            .await;
        redis_pool
            .hset(&cookie, "info", serde_json::json!(info).to_string())
            .await;
        redis_pool.expire(&cookie, 24 * 3600).await;
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
    pub async fn insert(self) -> Result<String, String> {
        NewUser::new(self).insert().await
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangePassword {
    pub old_password: String,
    pub new_password: String,
}

impl ChangePassword {
    pub async fn change_password(&self, cookie: &str) -> Result<u64, String> {
        let info = serde_json::from_str::<UserInfo>(
            &get_redis().hget::<String>(cookie, "info").await.unwrap(),
        )
        .unwrap();

        if !self.verification(info.id).await {
            return Err("Verification error".to_string());
        }

        let salt = random_string(6);
        let password = sha3_256_encode(get_password(&self.new_password) + &salt);

        sqlx::query!(
            r#"UPDATE users SET password = $1, salt = $2 WHERE id = $3"#,
            password,
            salt,
            info.id
        )
        .execute(get_postgres())
        .await
        .map(|r| r.rows_affected())
        .map_err(|e| format!("{}", e))
    }

    async fn verification(&self, id: Uuid) -> bool {
        #[derive(sqlx::FromRow)]
        struct Old {
            password: String,
            salt: String,
        }
        let old_user = sqlx::query_as!(
            Old,
            r#"SELECT password, salt FROM users
            WHERE id = $1"#,
            id
        )
        .fetch_one(get_postgres())
        .await;
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
    pub async fn edit_user(self, cookie: &str) -> Result<u64, String> {
        let redis_pool = get_redis();
        let info = serde_json::from_str::<UserInfo>(
            &redis_pool.hget::<String>(cookie, "info").await.unwrap(),
        )
        .unwrap();
        let res = sqlx::query_as!(
            UserInfo,
            r#"UPDATE users SET nickname = $1, say = $2, email = $3 WHERE id = $4
            RETURNING id, account, nickname, groups, say, email, create_time, github"#,
            self.nickname,
            self.say,
            self.email,
            info.id,
        )
        .fetch_one(get_postgres())
        .await;
        match res {
            Ok(data) => {
                redis_pool
                    .hset(cookie, "info", serde_json::json!(data).to_string())
                    .await;
                Ok(1)
            }
            Err(err) => Err(format!("{}", err)),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginUser {
    account: String,
    password: String,
    remember: bool,
}

impl LoginUser {
    pub async fn verification(&self, max_age: &Option<i64>) -> Result<String, String> {
        let res = sqlx::query_as!(
            Users,
            r#"SELECT * FROM users WHERE disabled = 0 AND account = $1"#,
            self.account
        )
        .fetch_one(get_postgres())
        .await;
        match res {
            Ok(data) => {
                if data.password == sha3_256_encode(get_password(&self.password) + &data.salt) {
                    let ttl = match *max_age {
                        Some(t) => t * 3600,
                        None => 24 * 60 * 60,
                    };

                    let redis_pool = get_redis();
                    let cookie = sha3_256_encode(random_string(8));
                    redis_pool
                        .hset(&cookie, "login_time", Local::now().timestamp())
                        .await;
                    redis_pool
                        .hset(
                            &cookie,
                            "info",
                            serde_json::json!(data.into_user_info()).to_string(),
                        )
                        .await;
                    redis_pool.expire(&cookie, ttl).await;
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

    pub async fn sign_out(cookies: &str) -> bool {
        get_redis().del(cookies).await
    }

    pub async fn login_with_github(
        github: String,
        nickname: String,
        account: String,
        token: &str,
    ) -> Result<String, String> {
        let ttl = 24 * 60 * 60;
        let redis_pool = get_redis();
        match UserInfo::view_user_with_github(&github).await {
            // github already exists
            Ok(data) => {
                let cookie = sha3_256_encode(random_string(8));
                redis_pool
                    .hset(&cookie, "login_time", Local::now().timestamp())
                    .await;
                redis_pool
                    .hset(&cookie, "info", serde_json::json!(data).to_string())
                    .await;
                redis_pool.expire(&cookie, ttl).await;
                Ok(cookie)
            }
            Err(_) => {
                let email = match get_github_primary_email(token).await {
                    Ok(data) => data,
                    Err(e) => return Err(e),
                };

                match UserInfo::view_user_with_email(&email).await {
                    // Account already exists but not linked
                    Ok(mut data) => {
                        match sqlx::query!(
                            r#"UPDATe users SET github = $1 WHERE id = $2"#,
                            github,
                            data.id
                        )
                        .execute(get_postgres())
                        .await
                        {
                            Ok(_) => {
                                data.github = Some(github);
                                let cookie = sha3_256_encode(random_string(8));
                                redis_pool
                                    .hset(&cookie, "login_time", Local::now().timestamp())
                                    .await;
                                redis_pool
                                    .hset(&cookie, "info", serde_json::json!(data).to_string())
                                    .await;
                                redis_pool.expire(&cookie, ttl).await;
                                Ok(cookie)
                            }
                            Err(err) => Err(format!("{}", err)),
                        }
                    }
                    // sign up
                    Err(_) => {
                        NewUser::new_with_github(email, github, account, nickname)
                            .insert()
                            .await
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Users {
    pub id: Uuid,
    pub account: String,
    pub github: Option<String>,
    pub password: String,
    pub salt: String,
    pub groups: i16,
    pub nickname: String,
    pub say: Option<String>,
    pub email: String,
    pub disabled: i16,
    pub create_time: NaiveDateTime,
}

impl Users {
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
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangePermission {
    pub id: Uuid,
    pub permission: i16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DisabledUser {
    id: Uuid,
    disabled: i16,
}
