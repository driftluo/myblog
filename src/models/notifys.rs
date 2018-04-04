use super::super::RedisPool;

use serde_json;
use std::sync::Arc;
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
    pub fn cache(&self, redis_pool: &Arc<RedisPool>) {
        let content = serde_json::to_string(self).unwrap();
        let notify_key = format!(
            "notify:{}:{}",
            self.article_id.hyphenated().to_string(),
            self.user_id.hyphenated().to_string()
        );
        // remove old value
        redis_pool.lrem(&notify_key, 0, &content);
        // put new value to list top
        redis_pool.lpush(&notify_key, &content);
        // set expire time 15 day or increase expire time to 15 day
        const EXPIRE_TIME: i64 = 5 * 24 * 3600;
        redis_pool.expire(&notify_key, EXPIRE_TIME);
        // limit list size to 100
        redis_pool.ltrim(&notify_key, 0, 10);
    }

    /// Get all the notifications about the user
    pub fn get_notifys(user_id: Uuid, redis_pool: &Arc<RedisPool>) -> Option<Vec<UserNotify>> {
        let pattern = format!("notify:*:{}", user_id.hyphenated().to_string());
        let mut notify = Vec::new();

        for notify_key in redis_pool.keys(&pattern) {
            let notifys: Vec<String> = redis_pool.lrange(&notify_key, 0, -1);
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
    pub fn remove_notifys_with_article_and_user(
        user_id: Uuid,
        article_id: Uuid,
        redis_pool: &Arc<RedisPool>,
    ) {
        let notify_key = format!(
            "notify:{}:{}",
            article_id.hyphenated().to_string(),
            user_id.hyphenated().to_string()
        );
        redis_pool.del(notify_key);
    }

    /// Remove the notification of the specified article, e.g use on remove the specified article
    pub fn remove_with_article(article_id: Uuid, redis_pool: &Arc<RedisPool>) {
        let pattern = format!("notify:{}*", article_id.hyphenated().to_string());
        redis_pool.del(redis_pool.keys(&pattern));
    }

    /// Remove the notification of the user, e.g use on remove the user
    pub fn remove_with_user(user_id: Uuid, redis_pool: &Arc<RedisPool>) {
        let pattern = format!("notify:*:{}", user_id.hyphenated().to_string());
        redis_pool.del(redis_pool.keys(&pattern));
    }
}
