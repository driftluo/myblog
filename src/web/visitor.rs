use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use sapper_std::{ Context, render, SessionVal, PathParams };
use uuid::Uuid;

use super::super::{ TagCount, Postgresql, Redis, AdminSession, UserSession, ArticlesWithTag,
                    user_verification_cookie, admin_verification_cookie, UserInfo, visitor_log };

pub struct ArticleWeb;

impl ArticleWeb {
    fn index(req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        match TagCount::view_tag_count(&pg_pool) {
            Ok(data) => web.add("tags", &data),
            Err(err) => println!("No tags, {}", err)
        }
        let admin_cookies_status = req.ext().get::<AdminSession>().unwrap();
        web.add("admin", admin_cookies_status);
        res_html!("visitor/index.html", web)
    }

    fn about(req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();
        let admin_cookies_status = req.ext().get::<AdminSession>().unwrap();
        web.add("admin", admin_cookies_status);
        res_html!("visitor/about.html", web)
    }

    fn list(req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();
        let admin_cookies_status = req.ext().get::<AdminSession>().unwrap();
        web.add("admin", admin_cookies_status);
        res_html!("visitor/list.html", web)
    }

    fn home(req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();
        let user_cookies_status = req.ext().get::<UserSession>().unwrap();
        let admin_cookies_status = req.ext().get::<AdminSession>().unwrap();
        web.add("admin", admin_cookies_status);
        match user_cookies_status {
           &false => res_html!("visitor/login.html", web),
            &true => res_html!("visitor/user.html", web)
        }
    }

    fn user(req: &mut Request) -> SapperResult<Response> {
        let params = get_path_params!(req);
        let article_id: Uuid = t_param!(params, "id").clone().parse().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let admin_cookies_status = req.ext().get::<AdminSession>().unwrap();
        let mut web = Context::new();
        web.add("admin", admin_cookies_status);
        match UserInfo::view_user(&pg_pool, article_id) {
            Ok(ref data) => web.add("user", data),
            Err(err) => println!("{}", err)
        };
        res_html!("visitor/user_info.html", web)
    }

    fn article_view(req: &mut Request) -> SapperResult<Response> {
        let params = get_path_params!(req);
        let article_id: Uuid = t_param!(params, "id").clone().parse().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let admin_cookies_status = req.ext().get::<AdminSession>().unwrap();
        let user_cookies_status = req.ext().get::<UserSession>().unwrap();
        let mut web = Context::new();
        web.add("admin", admin_cookies_status);
        web.add("user", user_cookies_status);
        match ArticlesWithTag::query_article(&pg_pool, article_id, false) {
            Ok(ref data) => web.add("article", data),
            Err(err) => println!("{}", err)
        }
        res_html!("visitor/article_view.html", web)
    }
}

impl SapperModule for ArticleWeb {
    #[allow(unused_assignments)]
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        let mut user_status = false;
        let mut admin_status = false;
        {
            let cookie = req.ext().get::<SessionVal>();
            let redis_pool = req.ext().get::<Redis>().unwrap();
            user_status = user_verification_cookie(cookie, redis_pool);
            admin_status = admin_verification_cookie(cookie, redis_pool);
        }
        req.ext_mut().insert::<UserSession>(user_status);
        req.ext_mut().insert::<AdminSession>(admin_status);

        Ok(())
    }

    fn after(&self, req: &Request, _res: &mut Response) -> SapperResult<()> {
        let redis_pool = req.ext().get::<Redis>().unwrap();
        visitor_log(req, redis_pool);
        Ok(())
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        // http get /
        router.get("/", ArticleWeb::index);

        // http get /index
        router.get("/index", ArticleWeb::index);

        // http get /about
        router.get("/about", ArticleWeb::about);

        // http get /list
        router.get("/list", ArticleWeb::list);

        // http get /login
        router.get("/home", ArticleWeb::home);

        router.get("/user/:id", ArticleWeb::user);

        router.get("/article/:id", ArticleWeb::article_view);

        Ok(())
    }
}
