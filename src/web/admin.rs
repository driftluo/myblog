use salvo::{
    http::{HttpError, StatusCode},
    prelude::{async_trait, fn_handler},
    Depot, Request, Response, Router, Writer,
};
use tera::Context;

use crate::{
    models::{articles::ArticlesWithTag, tag::Tags},
    utils::{from_code, parse_query},
    web::render,
    Routers, PERMISSION, WEB,
};

#[fn_handler]
async fn block_no_admin(depot: &mut Depot) -> Result<(), HttpError> {
    match depot.try_borrow::<_, Option<i16>>(PERMISSION) {
        Some(Some(0)) => Ok(()),
        _ => Err(from_code(StatusCode::FORBIDDEN, "No permission")),
    }
}

#[fn_handler]
async fn admin(depot: &mut Depot, res: &mut Response) {
    let web = depot.take::<_, Context>(WEB);

    render(res, "admin/admin.html", &web)
}

#[fn_handler]
async fn admin_list(depot: &mut Depot, res: &mut Response) {
    let web = depot.take::<_, Context>(WEB);

    render(res, "admin/admin_list.html", &web)
}

#[fn_handler]
async fn new_(depot: &mut Depot, res: &mut Response) {
    let mut web = depot.take::<_, Context>(WEB);

    match Tags::view_list_tag().await {
        Ok(tags_) => web.insert("tags", &tags_),
        Err(_) => (),
    }

    render(res, "admin/article_new.html", &web);
}

#[fn_handler]
async fn admin_view_article(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), HttpError> {
    let id = parse_query::<uuid::Uuid>(&req, "id")?;
    let mut web = depot.take::<_, Context>(WEB);

    match ArticlesWithTag::query_article(id, true).await {
        Ok(data) => web.insert("article", &data),
        // no possible, unless the admin does something strange
        Err(e) => println!("{}", e),
    }

    render(res, "admin/article_view.html", &web);
    Ok(())
}

#[fn_handler]
async fn article_edit(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), HttpError> {
    let id = parse_query::<String>(&req, "id")?;
    let mut web = depot.take::<_, Context>(WEB);
    web.insert("id", &id);

    match Tags::view_list_tag().await {
        Ok(tags_) => web.insert("tags", &tags_),
        Err(_) => (),
    }

    render(res, "admin/article_edit.html", &web);
    Ok(())
}

#[fn_handler]
async fn tags(depot: &mut Depot, res: &mut Response) {
    let web = depot.take::<_, Context>(WEB);

    render(res, "admin/tags.html", &web)
}

#[fn_handler]
async fn users(depot: &mut Depot, res: &mut Response) {
    let web = depot.take::<_, Context>(WEB);

    render(res, "admin/users.html", &web)
}

#[fn_handler]
async fn visitor_ip_log(depot: &mut Depot, res: &mut Response) {
    let web = depot.take::<_, Context>(WEB);

    render(res, "admin/ip.html", &web)
}

#[fn_handler]
async fn notify(depot: &mut Depot, res: &mut Response) {
    let web = depot.take::<_, Context>(WEB);

    render(res, "admin/notify.html", &web)
}

pub struct Admin;

impl Routers for Admin {
    fn build(self) -> Vec<Router> {
        vec![Router::new()
            .path("admin")
            .before(block_no_admin)
            // http {ip}/admin
            .get(admin)
            // http {ip}/admin/new
            .push(Router::new().path("new").get(new_))
            // http {ip}/admin/list
            .push(Router::new().path("list").get(admin_list))
            // http {ip}/admin/article/view?id=xxx
            .push(Router::new().path("article/view").get(admin_view_article))
            // http {ip}/admin/article/edit?id=xxx
            .push(Router::new().path("article/edit").get(article_edit))
            // http {ip}/admin/tags
            .push(Router::new().path("tags").get(tags))
            // http {ip}/admin/users
            .push(Router::new().path("users").get(users))
            // http {ip}/admin/ip
            .push(Router::new().path("ip").get(visitor_ip_log))
            // http {ip}/admin/notify
            .push(Router::new().path("notify").get(notify))]
    }
}
