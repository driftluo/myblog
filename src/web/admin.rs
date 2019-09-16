use sapper::{
    Error as SapperError, Request, Response, Result as SapperResult, SapperModule, SapperRouter,
};
use sapper_std::{render, QueryParams};
use uuid::Uuid;

use super::super::{ArticlesWithTag, Permissions, Postgresql, Tags, WebContext};

pub struct Admin;

impl Admin {
    fn admin(req: &mut Request) -> SapperResult<Response> {
        let web = req.ext().get::<WebContext>().unwrap().clone();
        res_html!("admin/admin.html", web)
    }

    fn admin_list(req: &mut Request) -> SapperResult<Response> {
        let web = req.ext().get::<WebContext>().unwrap().clone();
        res_html!("admin/admin_list.html", web)
    }

    fn new_(req: &mut Request) -> SapperResult<Response> {
        let mut web = req.ext().get::<WebContext>().unwrap().clone();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        match Tags::view_list_tag(&pg_pool) {
            Ok(ref data) => web.add("tags", data),
            Err(err) => println!("No tags, {}", err),
        }
        res_html!("admin/article_new.html", web)
    }

    fn admin_view_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let article_id = t_param_parse!(params, "id", Uuid);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let mut web = req.ext().get::<WebContext>().unwrap().clone();

        match ArticlesWithTag::query_article(&pg_pool, article_id, true) {
            Ok(ref data) => web.add("article", data),
            Err(err) => println!("{}", err),
        }
        res_html!("admin/article_view.html", web)
    }

    fn article_edit(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let article_id = t_param_parse!(params, "id", String);
        let mut web = req.ext().get::<WebContext>().unwrap().clone();
        web.add("id", &article_id);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        match Tags::view_list_tag(&pg_pool) {
            Ok(ref data) => web.add("tags", data),
            Err(err) => println!("No tags, {}", err),
        }
        res_html!("admin/article_edit.html", web)
    }

    fn tags(req: &mut Request) -> SapperResult<Response> {
        let web = req.ext().get::<WebContext>().unwrap().clone();
        res_html!("admin/tags.html", web)
    }

    fn users(req: &mut Request) -> SapperResult<Response> {
        let web = req.ext().get::<WebContext>().unwrap().clone();
        res_html!("admin/users.html", web)
    }

    fn visitor_ip_log(req: &mut Request) -> SapperResult<Response> {
        let web = req.ext().get::<WebContext>().unwrap().clone();
        res_html!("admin/ip.html", web)
    }

    fn notify(req: &mut Request) -> SapperResult<Response> {
        let web = req.ext().get::<WebContext>().unwrap().clone();
        res_html!("admin/notify.html", web)
    }
}

impl SapperModule for Admin {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        let permission = req.ext().get::<Permissions>().unwrap();
        match *permission {
            Some(0) => Ok(()),
            _ => Err(SapperError::TemporaryRedirect("/home".to_owned())),
        }
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        // http get /admin
        router.get("/admin", Admin::admin);

        router.get("/admin/list", Admin::admin_list);

        router.get("/admin/new", Admin::new_);

        router.get("/admin/article/view", Admin::admin_view_article);

        router.get("/admin/article/edit", Admin::article_edit);

        router.get("/admin/tags", Admin::tags);

        router.get("/admin/users", Admin::users);

        router.get("/admin/ip", Admin::visitor_ip_log);

        router.get("/admin/notify", Admin::notify);

        Ok(())
    }
}
