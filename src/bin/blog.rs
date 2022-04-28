use bytes::BytesMut;
use new_blog::{
    api::{init_page_size, AdminArticle, AdminUser, ChartData, Tag, User, Visitor},
    db_wrapper::{create_pg_pool, create_redis_pool},
    utils::get_identity_and_web_context,
    web::{Admin, ArticleWeb},
    Routers, PERMISSION, WEB,
};
use salvo::{
    extra::serve_static::DirHandler,
    http::{header, response::Body, StatusCode},
    listener::TcpListener,
    prelude::{async_trait, fn_handler},
    routing::FlowCtrl,
    Depot, Request, Response, Router, Server,
};
use tracing::{Instrument, Level};
use tracing_subscriber::FmtSubscriber;

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    dotenv::dotenv().ok();
    let listen_port = ::std::env::var("LISTEN_PORT")
        .expect("LISTEN_PORT must be set")
        .parse::<u16>()
        .unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        // load lua to redis
        create_redis_pool(Some("lua/visitor_log.lua")).await;
        create_pg_pool().await;
        init_page_size().await;

        let root = Router::new()
            .hoop(global)
            .append(ArticleWeb.build())
            .append(Admin.build())
            .append(AdminUser.build())
            .append(ChartData.build())
            .append(Tag.build())
            .append(AdminArticle.build())
            .append(User.build())
            .append(Visitor.build())
            .push(Router::new().path("robots.txt").get(robot))
            .push(
                Router::new()
                    .path(r#"<*path:/(js|css|images)/.+\.(js|css|webp)/>"#)
                    .get(DirHandler::new("static")),
            );

        Server::new(TcpListener::bind(([127, 0, 0, 1], listen_port)))
            .serve(root)
            .instrument(tracing::info_span!("listen start"))
            .await
    });
}

#[fn_handler]
async fn global(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    let (identity, web) = get_identity_and_web_context(req, depot).await;

    depot.insert(PERMISSION, identity);
    depot.insert(WEB, web);
    ctrl.call_next(req, depot, res).await;
}

#[fn_handler]
async fn robot(res: &mut Response) {
    const ROBOT: &str = r#"User-Agent: *
Allow: /
Allow: /*.css
Allow: /*.js

Sitemap:https://www.driftluo.com/rss
"#;
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    res.set_body(Some(Body::Bytes(BytesMut::from(ROBOT))));
    res.set_status_code(StatusCode::OK)
}
