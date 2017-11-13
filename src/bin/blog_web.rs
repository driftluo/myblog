extern crate sapper;
extern crate sapper_std;
extern crate blog;

use sapper::{ SapperApp, SapperAppShell, Request, Response, Result as SapperResult };
use blog::{ ArticleWeb };

struct WebApp;

impl SapperAppShell for WebApp {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        sapper_std::init(req)?;
        Ok(())
    }

    fn after(&self, req: &Request, res: &mut Response) -> SapperResult<()> {
        sapper_std::finish(req, res)?;
        Ok(())
    }
}

fn main() {
    let mut app = SapperApp::new();
    app.address("127.0.0.1")
        .port(8080)
        .with_shell(Box::new(WebApp))
        .add_module(Box::new(ArticleWeb))
        .static_service(true);

    println!("Start listen on {}", "127.0.0.1:8080");
    app.run_http();
}
