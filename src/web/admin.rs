use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use sapper_std::{ Context, render, SessionVal };

use super::super::{ admin_verification_cookie, Redis };

pub struct Admin;

impl Admin {
    fn admin(_req: &mut Request) -> SapperResult<Response> {
        let web = Context::new();
        res_html!("admin/admin.html", web)
    }

    fn admin_list(_req: &mut Request) -> SapperResult<Response> {
        let web = Context::new();
        res_html!("admin/admin_list.html", web)
    }

    fn new(_req: &mut Request) -> SapperResult<Response> {
        let web = Context::new();
        res_html!("admin/article_editor.html", web)
    }
}

impl SapperModule for Admin {
    #[allow(unused_assignments)]
    fn before(&self, req: &mut Request) -> SapperResult<Option<Response>> {
        let cookie = req.ext().get::<SessionVal>();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        match admin_verification_cookie(cookie, redis_pool) {
            true => { Ok(None) }
            false => {
                let res = json!({
                    "status": false,
                    "error": String::from("Verification error")
                });
                res_json!(res, true)
            }
        }
    }

    fn after(&self, _req: &Request, _res: &mut Response) -> SapperResult<()> {
        Ok(())
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        // http get /admin
        router.get("/admin", Admin::admin);

        router.get("/admin/list", Admin::admin_list);

        router.get("/admin/new", Admin::new);

        Ok(())
    }
}
