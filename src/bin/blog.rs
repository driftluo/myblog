extern crate sapper;
extern crate blog;
extern crate sapper_std;

use sapper::{ SapperApp, SapperAppShell, Request, Response, Result };
use blog::Article;

struct MyApp;

impl SapperAppShell for MyApp {
    fn before(&self, req: &mut Request) -> Result<()> {
        sapper_std::init(req)?;
        Ok(())
    }

    fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
        sapper_std::finish(req, res)?;
        Ok(())
    }
}

fn main() {
    let mut app = SapperApp::new();
    app.address("127.0.0.1")
        .port(8888)
        .with_shell(Box::new(MyApp))
        .add_module(Box::new(Article))
        .static_service(false);

    println!("Start listen on {}", "127.0.0.1:8888");
    app.run_http();
}
