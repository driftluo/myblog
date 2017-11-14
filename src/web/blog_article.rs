use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use sapper_std::{ Context, render };

use super::super::{ TagCount, establish_connection };

pub struct ArticleWeb;

impl ArticleWeb {
    fn index(_req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();
        let conn = establish_connection();
        match TagCount::view_tag_count(&conn) {
            Ok(data) => web.add("tags", &data),
            Err(err) => println!("No tags, {}", err)
        }
    res_html!("index.html", web)
    }
}

impl SapperModule for ArticleWeb {
    fn before(&self, _req: &mut Request) -> SapperResult<()> {
        Ok(())
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
