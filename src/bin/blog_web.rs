extern crate blog;
extern crate sapper;
extern crate sapper_std;

use blog::{create_pg_pool, create_redis_pool, get_identity_and_web_context, Admin, ArticleWeb,
           Permissions, Postgresql, Redis, WebContext};
use sapper::{Request, Response, Result as SapperResult, SapperApp, SapperAppShell};
use std::sync::Arc;

struct WebApp;

impl SapperAppShell for WebApp {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        sapper_std::init(req, Some("blog_session"))?;
        let (identity, web) = get_identity_and_web_context(req);
        req.ext_mut().insert::<Permissions>(identity);
        req.ext_mut().insert::<WebContext>(web);
        Ok(())
    }

    fn after(&self, req: &Request, res: &mut Response) -> SapperResult<()> {
        sapper_std::finish(req, res)?;
        Ok(())
    }
}

fn main() {
    let redis_pool = Arc::new(create_redis_pool(Some("lua/visitor_log.lua")));
    let pg_pool = create_pg_pool();
    let mut app = SapperApp::new();
    app.address("127.0.0.1")
        .port(8080)
        .init_global(Box::new(move |req: &mut Request| {
            req.ext_mut().insert::<Redis>(redis_pool.clone());
            req.ext_mut().insert::<Postgresql>(pg_pool.clone());
            Ok(())
        }))
        .with_shell(Box::new(WebApp))
        .add_module(Box::new(ArticleWeb))
        .add_module(Box::new(Admin))
        .static_service(true);

    println!("Start listen on {}", "127.0.0.1:8080");
    app.run_http();
}
