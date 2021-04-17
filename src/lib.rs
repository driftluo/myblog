#![allow(dead_code)]
pub mod api;
pub mod db_wrapper;
pub mod models;
pub mod utils;
pub mod web;

pub trait Routers {
    fn build(self) -> Vec<salvo::Router>;
}

pub const PERMISSION: &str = "permission";
pub const WEB: &str = "web";
pub const COOKIE: &str = "cookie";
pub const USER_INFO: &str = "user_info";
