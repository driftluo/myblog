extern crate sapper;
extern crate blog;
extern crate sapper_std;

use sapper::{ SapperApp, SapperAppShell, Request, Response, Result as SapperResult };
use blog::{ AdminArticle, Visitor, AdminUser, User, Tag, Redis, create_redis_pool, create_pg_pool, Postgresql };
use std::sync::Arc;

struct ApiApp;

impl SapperAppShell for ApiApp {
    fn before(&self, req: &mut Request) -> SapperResult<Option<Response>> {
        sapper_std::init(req, "blog_session")?;
        Ok(None)
    }

    fn after(&self, req: &Request, res: &mut Response) -> SapperResult<()> {
        sapper_std::finish(req, res)?;
        Ok(())
    }
}

fn main() {
    let redispool = Arc::new(create_redis_pool());
    let pg_pool = create_pg_pool();
    let mut app = SapperApp::new();
    app.address("127.0.0.1")
        .port(8888)
        .init_global(
        Box::new(move |req: &mut Request| {
                req.ext_mut().insert::<Redis>(redispool.clone());
                req.ext_mut().insert::<Postgresql>(pg_pool.clone());
                Ok(())
            })
        )
        .with_shell(Box::new(ApiApp))
        .add_module(Box::new(Visitor))
        .add_module(Box::new(User))
        .add_module(Box::new(AdminArticle))
        .add_module(Box::new(Tag))
        .add_module(Box::new(AdminUser))
        .static_service(false);

    println!("Start listen on {}", "127.0.0.1:8888");
    app.run_http();
}
