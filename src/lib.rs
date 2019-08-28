#![recursion_limit = "128"]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate sapper_std;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod api;
pub mod models;
pub mod schema;
pub mod util;
pub mod web;

pub use api::AdminArticle;
pub use api::AdminUser;
pub use api::ChartData;
pub use api::Tag;
pub use api::User;
pub use api::Visitor;
pub(crate) use models::UserNotify;
pub(crate) use models::{
    ArticleList, ArticlesWithTag, EditArticle, ModifyPublish, NewArticle, PublishedStatistics,
};
pub(crate) use models::{
    ChangePassword, ChangePermission, DisabledUser, EditUser, LoginUser, RegisteredUser, UserInfo,
    Users,
};
pub(crate) use models::{Comments, DeleteComment, NewComments};
pub(crate) use models::{NewTag, TagCount, Tags};
pub(crate) use schema::{article_tag_relation, article_with_tag, articles, comments, tags, users};
#[cfg(not(feature = "monitor"))]
pub(crate) use util::visitor_log;
pub use util::{create_pg_pool, Postgresql};
pub use util::{
    create_redis_pool, get_identity_and_web_context, Permissions, Redis, RedisPool, WebContext,
};
pub(crate) use util::{
    get_github_account_nickname_address, get_github_primary_email, get_github_token,
};
pub(crate) use util::{get_password, markdown_render, random_string, sha3_256_encode};
pub use web::{Admin, ArticleWeb};
