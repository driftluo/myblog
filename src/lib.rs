#![recursion_limit="128"]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate dotenv;
extern crate chrono;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate serde;
extern crate sapper;
#[macro_use]
extern crate sapper_std;
extern crate rand;
extern crate md5;


pub mod schema;
pub mod models;
pub mod blog_article;
pub mod blog_user;
pub mod util;

pub(crate) use schema::{ articles, users, article_with_tag, tags, article_tag_relation };
pub(crate) use models::{ NewArticle, Articles, ArticleList, ModifyPublish, EditArticle };
pub(crate) use models::{ UserInfo, Users, NewUser, ChangePassword, RegisteredUser };
pub(crate) use util::{ md5_encode, random_string };
pub use blog_article::Article;
pub use blog_user::User;

use std::env;
use diesel::pg::PgConnection;
use diesel::Connection;

pub(crate) fn establish_connection() -> PgConnection {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}
