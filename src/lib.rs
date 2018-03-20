#![recursion_limit = "128"]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate ammonia;
extern crate chrono;
extern crate comrak;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_infer_schema;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_redis;
extern crate rand;
extern crate redis;
extern crate sapper;
#[macro_use]
extern crate sapper_std;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tiny_keccak;
extern crate uuid;

pub mod schema;
pub mod models;
pub mod util;
pub mod api;
pub mod web;

pub(crate) use schema::{article_tag_relation, article_with_tag, articles, comments, tags, users};
pub(crate) use models::{ArticleList, ArticlesWithTag, EditArticle, ModifyPublish, NewArticle,
                        PublishedStatistics};
pub(crate) use models::{ChangePassword, ChangePermission, DisabledUser, EditUser, LoginUser,
                        NewUser, RegisteredUser, UserInfo, Users};
pub(crate) use models::{NewTag, TagCount, Tags};
pub(crate) use models::{Comments, DeleteComment, NewComments};
pub(crate) use util::{admin_verification_cookie, get_password, markdown_render, random_string,
                      user_verification_cookie, AdminSession, UserSession, sha3_256_encode};
pub(crate) use util::visitor_log;
pub use util::{create_redis_pool, Redis, RedisPool};
pub use util::{create_pg_pool, Postgresql};
pub use api::Visitor;
pub use api::User;
pub use api::AdminArticle;
pub use api::Tag;
pub use api::AdminUser;
pub use api::ChartData;
pub use web::{Admin, ArticleWeb};
