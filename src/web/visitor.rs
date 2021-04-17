use salvo::{
    http::{HttpError, StatusCode},
    prelude::{async_trait, fn_handler},
    Depot, Request, Response, Router, Writer,
};
use tera::Context;
use uuid::Uuid;

use crate::{
    db_wrapper::get_redis,
    models::{articles::ArticlesWithTag, notify::UserNotify, tag::TagCount, user::UserInfo},
    utils::{from_code, parse_last_path, visitor_log},
    web::render,
    Routers, COOKIE, PERMISSION, WEB,
};

#[tracing::instrument]
#[fn_handler]
async fn index(depot: &mut Depot, res: &mut Response) {
    let mut web = depot.take::<_, Context>(WEB);

    match TagCount::view_tag_count().await {
        Ok(data) => web.insert("tags", &data),
        Err(e) => tracing::info!("can't get tags with {}", e),
    }

    render(res, "visitor/index.html", &web)
}

#[fn_handler]
async fn about(depot: &mut Depot, res: &mut Response) {
    let web = depot.take::<_, Context>(WEB);

    render(res, "visitor/about.html", &web)
}

#[fn_handler]
async fn list(depot: &mut Depot, res: &mut Response) {
    let web = depot.take::<_, Context>(WEB);

    render(res, "visitor/list.html", &web)
}

#[fn_handler]
async fn home(depot: &mut Depot, res: &mut Response) {
    let web = depot.take::<_, Context>(WEB);

    let permission = depot.take::<_, Option<i16>>(PERMISSION);

    match permission {
        Some(_) => render(res, "visitor/user.html", &web),
        None => render(res, "visitor/login.html", &web),
    }
}

#[fn_handler]
async fn user(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), HttpError> {
    let id = parse_last_path::<Uuid>(req)?;
    let mut web = depot.take::<_, Context>(WEB);

    match UserInfo::view_user(id).await {
        Ok(ref data) => {
            web.insert("user_info", data);
            render(res, "visitor/user_info.html", &web)
        }
        Err(_) => return Err(from_code(StatusCode::NOT_FOUND, "Query Param is Incorrect")),
    }
    Ok(())
}

#[fn_handler]
async fn article_view(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), HttpError> {
    let id = parse_last_path::<Uuid>(req)?;
    let mut web = depot.take::<_, Context>(WEB);

    match ArticlesWithTag::query_article(id, false).await {
        Ok(data) => {
            web.insert("article", &data);
            if let Some(cookie) = depot.try_take::<_, String>(COOKIE) {
                if let Ok(info) = get_redis().hget::<String>(&cookie, "info").await {
                    let info = serde_json::from_str::<UserInfo>(&info).unwrap();

                    UserNotify::remove_notifys_with_article_and_user(info.id, data.id).await;
                    let notify = UserNotify::get_notifys(info.id).await;
                    web.insert("notify", &notify);
                }
            }
            render(res, "visitor/article_view.html", &web)
        }
        Err(err) => return Err(from_code(StatusCode::NOT_FOUND, err)),
    }

    Ok(())
}

pub struct ArticleWeb;

impl Routers for ArticleWeb {
    fn build(self) -> Vec<Router> {
        vec![
            // http {ip}/index
            Router::new().before(visitor_log).get(index),
            Router::new().path("index").before(visitor_log).get(index),
            // http {ip}/about
            Router::new().path("about").get(about),
            // http {ip}/list
            Router::new().path("list").get(list),
            // http {ip}/home
            Router::new().path("home").get(home),
            // http {ip}/<id>
            Router::new()
                .path("user/<id:/[0-9a-z]{8}(-[0-9a-z]{4}){3}-[0-9a-z]{8}/>")
                .get(user),
            // http {ip}/article/<id>
            Router::new()
                .path("article/<id:/[0-9a-z]{8}(-[0-9a-z]{4}){3}-[0-9a-z]{8}/>")
                .get(article_view),
        ]
    }
}
