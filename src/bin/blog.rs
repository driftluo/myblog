use bytes::BytesMut;
use new_blog::{
    api::{init_page_size, AdminArticle, AdminUser, ChartData, Tag, User, Visitor},
    db_wrapper::{create_pg_pool, create_redis_pool},
    utils::get_identity_and_web_context,
    web::{Admin, ArticleWeb},
    Routers, PERMISSION, WEB,
};
use salvo::prelude::Listener;
use salvo::{
    conn::TcpListener,
    http::{header, ResBody, StatusCode},
    prelude::handler,
    routing::FlowCtrl,
    serve_static::StaticDir,
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
            .append(&mut ArticleWeb.build())
            .append(&mut Admin.build())
            .append(&mut AdminUser.build())
            .append(&mut ChartData.build())
            .append(&mut Tag.build())
            .append(&mut AdminArticle.build())
            .append(&mut User.build())
            .append(&mut Visitor.build())
            .push(Router::new().path("robots.txt").get(robot))
            .push(
                Router::new()
                    .path("{*path}")
                    .get(StaticDir::new(["static"]).exclude(|path| {
                        // Only allow specific file extensions for security
                        let allowed = [
                            ".css", ".js", ".png", ".jpg", ".jpeg", ".gif", ".ico", ".webp",
                            ".woff", ".woff2", ".svg",
                        ];
                        !allowed.iter().any(|ext| path.to_lowercase().ends_with(ext))
                    })),
            );

        let acceptor = TcpListener::new(format!("127.0.0.1:{}", listen_port))
            .bind()
            .await;
        Server::new(acceptor)
            .serve(root)
            .instrument(tracing::info_span!("listen start"))
            .await
    });
}

#[handler]
async fn global(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    let (identity, web) = get_identity_and_web_context(req, depot).await;

    depot.insert(PERMISSION, identity);
    depot.insert(WEB, web);
    ctrl.call_next(req, depot, res).await;
}

#[handler]
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
    res.body(ResBody::Once(BytesMut::from(ROBOT).freeze()));
    res.status_code(StatusCode::OK);
}
