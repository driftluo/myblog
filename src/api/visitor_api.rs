use sapper::{Request, Response, Result as SapperResult, SapperModule, SapperRouter};
use sapper_std::{set_cookie, JsonParams, PathParams, QueryParams, SessionVal};
use sapper::header::{ContentType, Location};
use sapper::status;
use serde_json;

use super::super::{ArticleList, ArticlesWithTag, Comments, LoginUser, Permissions, Postgresql,
                   Redis, RegisteredUser, UserInfo, get_github_token, get_github_account_nickname_address};
use uuid::Uuid;

pub struct Visitor;

impl Visitor {
    fn list_all_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let limit = t_param_parse!(params, "limit", i64);
        let offset = t_param_parse!(params, "offset", i64);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match ArticleList::query_list_article(&pg_pool, limit, offset, false) {
            Ok(data) => json!({
                    "status": true,
                    "data": data
                }),
            Err(err) => json!({
                    "status": false,
                    "error": err
                }),
        };
        res_json!(res)
    }

    fn list_all_article_filter_by_tag(req: &mut Request) -> SapperResult<Response> {
        let params = get_path_params!(req);
        let tag_id: Uuid = t_param!(params, "tag_id").clone().parse().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match ArticleList::query_with_tag(&pg_pool, tag_id) {
            Ok(data) => json!({
                    "status": true,
                    "data": data
                }),
            Err(err) => json!({
                    "status": false,
                    "error": err
                }),
        };
        res_json!(res)
    }

    fn list_comments(req: &mut Request) -> SapperResult<Response> {
        let path_params = get_path_params!(req);
        let article_id: Uuid = t_param!(path_params, "id").clone().parse().unwrap();
        let query_params = get_query_params!(req);
        let limit = t_param_parse!(query_params, "limit", i64);
        let offset = t_param_parse!(query_params, "offset", i64);

        let permission = req.ext().get::<Permissions>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let (user_id, admin) = match *permission {
            Some(0) => {
                let cookie = req.ext().get::<SessionVal>().unwrap();
                let info = serde_json::from_str::<UserInfo>(&UserInfo::view_user_with_cookie(
                    redis_pool,
                    cookie,
                )).unwrap();
                (Some(info.id), true)
            }
            Some(_) => {
                let cookie = req.ext().get::<SessionVal>().unwrap();
                let info = serde_json::from_str::<UserInfo>(&UserInfo::view_user_with_cookie(
                    redis_pool,
                    cookie,
                )).unwrap();
                (Some(info.id), false)
            }
            _ => (None, false),
        };
        let res = match Comments::query(&pg_pool, limit, offset, article_id) {
            Ok(data) => json!({
                    "status": true,
                    "data": data,
                    "admin": admin,
                    "user": user_id
                }),
            Err(err) => json!({
                    "status": false,
                    "error": err
                }),
        };
        res_json!(res)
    }

    fn view_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let article_id = t_param_parse!(params, "id", Uuid);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let res = match ArticlesWithTag::query_without_article(&pg_pool, article_id, false) {
            Ok(data) => json!({
                    "status": true,
                    "data": data
                }),
            Err(err) => json!({
                    "status": false,
                    "error": err
                }),
        };
        res_json!(res)
    }

    fn login(req: &mut Request) -> SapperResult<Response> {
        let body: LoginUser = get_json_params!(req);
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        let max_age: Option<i64> = if body.get_remember() {
            Some(24 * 90)
        } else {
            None
        };

        match body.verification(&pg_pool, redis_pool, &max_age) {
            Ok(cookies) => {
                let res = json!({
                    "status": true,
                });

                response.write_body(serde_json::to_string(&res).unwrap());

                let _ = set_cookie(
                    &mut response,
                    "blog_session".to_string(),
                    cookies,
                    None,
                    Some("/".to_string()),
                    None,
                    max_age,
                );
            }
            Err(err) => {
                let res = json!({
                    "status": false,
                    "error": format!("{}", err)
                });

                response.write_body(serde_json::to_string(&res).unwrap());
            }
        };

        Ok(response)
    }

    fn login_with_github(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let code = t_param_parse!(params, "code", String);

        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let token = get_github_token(&code)?;

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        let (account, nickname, github_address) = get_github_account_nickname_address(&token)?;
        match LoginUser::login_with_github(&pg_pool, redis_pool, github_address, nickname, account, &token) {
            Ok(cookie) => {
                let res = json!({
                    "status": true,
                });

                response.set_status(status::Found);
                response.write_body(serde_json::to_string(&res).unwrap());
                response.headers_mut().set(Location("/home".to_owned()));

                let _ = set_cookie(
                    &mut response,
                    "blog_session".to_string(),
                    cookie,
                    None,
                    Some("/".to_string()),
                    None,
                    Some(24),
                );
            }

            Err(err) => {
                let res = json!({
                    "status": false,
                    "error": format!("{}", err)
                });

                response.write_body(serde_json::to_string(&res).unwrap());
            }
        }

        Ok(response)
    }

    fn create_user(req: &mut Request) -> SapperResult<Response> {
        let body: RegisteredUser = get_json_params!(req);

        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        match body.insert(&pg_pool, redis_pool) {
            Ok(cookies) => {
                let res = json!({
                    "status": true,
                });

                response.write_body(serde_json::to_string(&res).unwrap());

                let _ = set_cookie(
                    &mut response,
                    "blog_session".to_string(),
                    cookies,
                    None,
                    Some("/".to_string()),
                    None,
                    Some(24),
                );
            }
            Err(err) => {
                let res = json!({
                    "status": false,
                    "error": format!("{}", err)
                });

                response.write_body(serde_json::to_string(&res).unwrap());
            }
        }
        Ok(response)
    }
}

impl SapperModule for Visitor {
    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        // http get /article/view_all limit==5 offset==0
        router.get("/article/view_all", Visitor::list_all_article);

        router.get(
            "/article/view_all/:tag_id",
            Visitor::list_all_article_filter_by_tag,
        );

        router.get("/article/view_comment/:id", Visitor::list_comments);

        // http get /article/view id==4
        router.get("/article/view", Visitor::view_article);

        router.get("/login_with_github", Visitor::login_with_github);

        // http post :8888/user/login account=admin password=admin
        router.post("/user/login", Visitor::login);

        // http post :8888/user/new account="k1234" password="1234" nickname="漂流" email="441594700@qq.com" say=""
        router.post("/user/new", Visitor::create_user);

        Ok(())
    }
}
