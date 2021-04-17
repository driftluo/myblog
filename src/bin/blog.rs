use new_blog::{
    api::{AdminArticle, AdminUser, ChartData, Tag, User, Visitor},
    db_wrapper::{create_pg_pool, create_redis_pool},
    utils::get_identity_and_web_context,
    web::{index, Admin, ArticleWeb},
    Routers, PERMISSION, WEB,
};
use salvo::{
    extra::serve::StaticDir,
    prelude::{async_trait, fn_handler},
    Depot, Request, Router, Server,
};
use tracing::Instrument;

fn main() {
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

        let root = Router::new()
            .before(global)
            .get(index)
            .append(ArticleWeb.build())
            .append(Admin.build())
            .append(AdminUser.build())
            .append(ChartData.build())
            .append(Tag.build())
            .append(AdminArticle.build())
            .append(User.build())
            .append(Visitor.build())
            .push(
                Router::new()
                    .path("<**path>")
                    .get(StaticDir::new("static/")),
            );

        Server::new(root)
            .bind(([127, 0, 0, 1], listen_port))
            .instrument(tracing::info_span!("listen on 127.0.0.1:{}", listen_port))
            .await
    });
}

#[fn_handler]
async fn global(req: &mut Request, depot: &mut Depot) {
    let (identity, web) = get_identity_and_web_context(req, depot).await;

    depot.insert(PERMISSION, identity);
    depot.insert(WEB, web);
}
