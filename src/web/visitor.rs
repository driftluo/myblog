use sapper::{
    Error as SapperError, Request, Response, Result as SapperResult, SapperModule, SapperRouter,
};
use sapper_std::{render, PathParams, SessionVal};
use serde_json;
use uuid::Uuid;

#[cfg(not(feature = "monitor"))]
use super::super::visitor_log;
use super::super::{
    ArticlesWithTag, Permissions, Postgresql, Redis, TagCount, UserInfo, UserNotify, WebContext,
};

pub struct ArticleWeb;

impl ArticleWeb {
    fn index(req: &mut Request) -> SapperResult<Response> {
        let mut web = req.ext().get::<WebContext>().unwrap().clone();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        match TagCount::view_tag_count(&pg_pool) {
            Ok(data) => web.add("tags", &data),
            Err(err) => println!("No tags, {}", err),
        }

        res_html!("visitor/index.html", web)
    }

    fn about(req: &mut Request) -> SapperResult<Response> {
        let web = req.ext().get::<WebContext>().unwrap().clone();

        res_html!("visitor/about.html", web)
    }

    fn list(req: &mut Request) -> SapperResult<Response> {
        let web = req.ext().get::<WebContext>().unwrap().clone();

        res_html!("visitor/list.html", web)
    }

    fn home(req: &mut Request) -> SapperResult<Response> {
        let web = req.ext().get::<WebContext>().unwrap().clone();
        let permission = req.ext().get::<Permissions>().unwrap();

        match *permission {
            Some(_) => res_html!("visitor/user.html", web),
            None => res_html!("visitor/login.html", web),
        }
    }

    /// Query other user information
    fn user(req: &mut Request) -> SapperResult<Response> {
        let params = get_path_params!(req);
        let user_id: Uuid = t_param!(params, "id").parse().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let mut web = req.ext().get::<WebContext>().unwrap().clone();

        match UserInfo::view_user(&pg_pool, user_id) {
            Ok(ref data) => web.add("user_info", data),
            Err(err) => println!("{}", err),
        };
        res_html!("visitor/user_info.html", web)
    }

    fn article_view(req: &mut Request) -> SapperResult<Response> {
        let params = get_path_params!(req);
        let article_id: Uuid = t_param!(params, "id").parse().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let mut web = req.ext().get::<WebContext>().unwrap().clone();

        match ArticlesWithTag::query_article(&pg_pool, article_id, false) {
            Ok(ref data) => {
                web.add("article", data);

                // Remove user's notify about this article
                req.ext().get::<SessionVal>().and_then(|cookie| {
                    if redis_pool.exists(cookie) {
                        let info = serde_json::from_str::<UserInfo>(
                            &redis_pool.hget::<String>(cookie, "info"),
                        )
                        .unwrap();
                        UserNotify::remove_notifys_with_article_and_user(
                            info.id,
                            data.id,
                            &redis_pool,
                        );
                        // Replace the original notification
                        let notifys = UserNotify::get_notifys(info.id, redis_pool);
                        web.add("notifys", &notifys);
                    };
                    Some(())
                });
                res_html!("visitor/article_view.html", web)
            }
            Err(err) => {
                println!("{}", err);
                Err(SapperError::NotFound)
            }
        }
    }
}

impl SapperModule for ArticleWeb {
    #[cfg(not(feature = "monitor"))]
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

        // http get /home
        router.get("/home", ArticleWeb::home);

        router.get("/user/:id", ArticleWeb::user);

        router.get("/article/:id", ArticleWeb::article_view);

        Ok(())
    }
}
