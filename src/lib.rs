#![allow(dead_code)]
pub mod api;
pub mod db_wrapper;
pub mod models;
pub mod utils;
pub mod web;

pub trait Routers {
    fn build(self) -> Vec<salvo::Router>;
}

pub const PERMISSION: &'static str = "permission";
pub const WEB: &'static str = "web";
pub const COOKIE: &'static str = "cookie";
pub const USER_INFO: &'static str = "user_info";
