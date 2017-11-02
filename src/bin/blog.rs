extern crate sapper;
extern crate blog;

use sapper::{ SapperApp };
use blog::Article;

fn main() {
    let mut app = SapperApp::new();
    app.address("127.0.0.1")
        .port(8888)
        .add_module(Box::new(Article));

    println!("Start listen on {}", "127.0.0.1:8888");
    app.run_http();
}
