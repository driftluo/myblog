use crate::db_wrapper::get_redis;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserNotify {
    pub user_id: Uuid,
    pub send_user_name: String,
    pub article_id: Uuid,
    pub article_title: String,
    pub notify_type: String,
}

impl UserNotify {
    /// Cache user's comment notify to redis
    pub async fn cache(&self) {
        let content = serde_json::to_string(self).unwrap();
        let notify_key = format!(
            "notify:{}:{}",
            self.article_id.to_hyphenated().to_string(),
            self.user_id.to_hyphenated().to_string()
        );

        let redis_pool = get_redis();
        // remove old value
        redis_pool.lrem(&notify_key, 0, &content).await;
        // put new value to list top
        redis_pool.lpush(&notify_key, &content).await;
        // set expire time 15 day or increase expire time to 15 day
        const EXPIRE_TIME: i64 = 5 * 24 * 3600;
        redis_pool.expire(&notify_key, EXPIRE_TIME).await;
        // limit list size to 100
        redis_pool.ltrim(&notify_key, 0, 10).await;
    }

    /// Get all the notifications about the user
    pub async fn get_notifys(user_id: Uuid) -> Option<Vec<UserNotify>> {
        let pattern = format!("notify:*:{}", user_id.to_hyphenated().to_string());
        let mut notify = Vec::new();
        let redis_pool = get_redis();

        for notify_key in redis_pool.keys(&pattern).await {
            let notifys: Vec<String> = redis_pool.lrange(&notify_key, 0, -1).await;
            let notifys: Vec<UserNotify> = notifys
                .iter()
                .map(|notify_string| {
                    let user_notify: UserNotify = serde_json::from_str(&notify_string).unwrap();
                    user_notify
                })
                .collect();
            notify.extend(notifys);
        }

        if notify.is_empty() {
            None
        } else {
            Some(notify)
        }
    }

    /// Remove the notification of the specified article specified user
    pub async fn remove_notifys_with_article_and_user(user_id: Uuid, article_id: Uuid) {
        let notify_key = format!(
            "notify:{}:{}",
            article_id.to_hyphenated().to_string(),
            user_id.to_hyphenated().to_string()
        );
        get_redis().del(notify_key).await;
    }

    /// Remove the notification of the specified article, e.g use on remove the specified article
    pub async fn remove_with_article(article_id: Uuid) {
        let pattern = format!("notify:{}*", article_id.to_hyphenated().to_string());
        let redis_pool = get_redis();
        let keys = redis_pool.keys(&pattern).await;
        if !keys.is_empty() {
            redis_pool.del(keys).await;
        }
    }

    /// Remove the notification of the user, e.g use on remove the user
    pub async fn remove_with_user(user_id: Uuid) {
        let pattern = format!("notify:*:{}", user_id.to_hyphenated().to_string());
        let redis_pool = get_redis();
        let keys = redis_pool.keys(&pattern).await;
        if !keys.is_empty() {
            redis_pool.del(keys).await;
        }
    }
}
