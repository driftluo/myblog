use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use sapper_std::{ Context, render, SessionVal };

use super::super::{ TagCount, Postgresql, Redis, Session, user_verification_cookie };

pub struct ArticleWeb;

impl ArticleWeb {
    fn index(req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        match TagCount::view_tag_count(&pg_pool) {
            Ok(data) => web.add("tags", &data),
            Err(err) => println!("No tags, {}", err)
        }
    res_html!("index.html", web)
    }
}

impl SapperModule for ArticleWeb {
    #[allow(unused_assignments)]
    fn before(&self, req: &mut Request) -> SapperResult<Option<Response>> {
        let mut status = false;
        {
            let cookie = req.ext().get::<SessionVal>();
            let redis_pool = req.ext().get::<Redis>().unwrap();
            status = user_verification_cookie(cookie, redis_pool);
        }
        req.ext_mut().insert::<Session>(status);

        Ok(None)
    }

    fn after(&self, _req: &Request, _res: &mut Response) -> SapperResult<()> {
        Ok(())
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        // http get /
        router.get("/", ArticleWeb::index);

        // http get /index
        router.get("/index", ArticleWeb::index);

        Ok(())
    }
}
