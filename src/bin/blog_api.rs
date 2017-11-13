extern crate sapper;
extern crate blog;
extern crate sapper_std;

use sapper::{ SapperApp, SapperAppShell, Request, Response, Result as SapperResult };
use blog::{ Article, User, Tag };

struct ApiApp;

impl SapperAppShell for ApiApp {
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
        .port(8888)
        .with_shell(Box::new(ApiApp))
        .add_module(Box::new(Article))
        .add_module(Box::new(User))
        .add_module(Box::new(Tag))
        .static_service(false);

    println!("Start listen on {}", "127.0.0.1:8888");
    app.run_http();
}
