extern crate sapper;
extern crate sapper_std;
extern crate blog;

use sapper::{ SapperApp, SapperAppShell, Request, Response, Result as SapperResult };
use blog::{ ArticleWeb, create_redis_pool, Redis, create_pg_pool, Postgresql };
use std::sync::Arc;

struct WebApp;

impl SapperAppShell for WebApp {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        sapper_std::init(req, "blog_session")?;
        Ok(())
    }

    fn after(&self, req: &Request, res: &mut Response) -> SapperResult<()> {
        sapper_std::finish(req, res)?;
        Ok(())
    }
}

fn main() {
    let redis_pool = Arc::new(create_redis_pool());
    let pg_pool = create_pg_pool();
    let mut app = SapperApp::new();
    app.address("127.0.0.1")
        .port(8080)
        .init_global(
            Box::new(move |req: &mut Request| {
                req.ext_mut().insert::<Redis>(redis_pool.clone());
                req.ext_mut().insert::<Postgresql>(pg_pool.clone());
                Ok(())
            })
        )
        .with_shell(Box::new(WebApp))
        .add_module(Box::new(ArticleWeb))
        .static_service(true);

    println!("Start listen on {}", "127.0.0.1:8080");
    app.run_http();
}
