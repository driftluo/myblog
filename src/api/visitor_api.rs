use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use sapper_std::{ QueryParams, PathParams, JsonParams, set_cookie, SessionVal };
use sapper::header::ContentType;
use serde_json;

use super::super::{ ArticlesWithTag, RegisteredUser, NewUser, sha3_256_encode, random_string, get_password,
                    ArticleList, Postgresql, Redis, LoginUser, Comments, AdminSession, UserSession,
                    user_verification_cookie, admin_verification_cookie, UserInfo };
use uuid::Uuid;

pub struct Visitor;

impl Visitor {
    fn list_all_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let limit = t_param_parse!(params, "limit", i64);
        let offset = t_param_parse!(params, "offset", i64);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match ArticleList::query_list_article(&pg_pool, limit, offset, false) {
            Ok(data) => {
                json!({
                    "status": true,
                    "data": data
                })
            }
            Err(err) => {
                json!({
                    "status": false,
                    "error": err
                })
            }
        };
        res_json!(res)
    }

    fn list_all_article_filter_by_tag(req: &mut Request) -> SapperResult<Response> {
        let params = get_path_params!(req);
        let tag_id: Uuid = t_param!(params, "tag_id").clone().parse().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match ArticleList::query_with_tag(&pg_pool, tag_id) {
            Ok(data) => {
                json!({
                    "status": true,
                    "data": data
                })
            }
            Err(err) => {
                json!({
                    "status": false,
                    "error": err
                })
            }
        };
        res_json!(res)
    }

    fn list_comments(req: &mut Request) -> SapperResult<Response> {
        let path_params = get_path_params!(req);
        let article_id: Uuid = t_param!(path_params, "id").clone().parse().unwrap();
        let query_params = get_query_params!(req);
        let limit = t_param_parse!(query_params, "limit", i64);
        let offset = t_param_parse!(query_params, "offset", i64);

        let admin_cookies_status = req.ext().get::<AdminSession>().unwrap();
        let user_cookies_status = req.ext().get::<UserSession>().unwrap();


        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let user_id = match user_cookies_status {
          &true => {
              let cookie = req.ext().get::<SessionVal>().unwrap();
              let info = serde_json::from_str::<UserInfo>(&UserInfo::view_user_with_cookie(redis_pool, cookie, admin_cookies_status)).unwrap();
              Some(info.id)
          },
            &false => {
                None
            }
        };
        let res = match Comments::query(&pg_pool, limit, offset, article_id) {
            Ok(data) => {
                json!({
                    "status": true,
                    "data": data,
                    "admin": admin_cookies_status,
                    "user": user_id
                })
            }
            Err(err) => {
                json!({
                    "status": false,
                    "error": err
                })
            }
        };
        res_json!(res)
    }

    fn view_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let article_id = t_param_parse!(params, "id", Uuid);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let res = match ArticlesWithTag::query_article(&pg_pool, article_id, false) {
            Ok(data) => {
                json!({
                    "status": true,
                    "data": data
                })
            }
            Err(err) => {
                json!({
                    "status": false,
                    "error": err
                })
            }
        };
        res_json!(res)
    }

    fn login(req: &mut Request) -> SapperResult<Response> {
        let body: LoginUser = get_json_params!(req);
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        let max_age: Option<i64> = match body.get_remember() {
            true => Some(24 * 90),
            false => None
        };

        match body.verification(&pg_pool, redis_pool, &max_age) {
            Ok(cookies) => {
                let res = json!({
                    "status": true,
                });

                response.write_body(serde_json::to_string(&res).unwrap());

                let _ = set_cookie(&mut response, "blog_session".to_string(), cookies,
                                   None, Some("/".to_string()), None, max_age);

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

    fn create_user(req: &mut Request) -> SapperResult<Response> {
        let mut body: RegisteredUser = get_json_params!(req);
        let salt = random_string(6);
        body.password = sha3_256_encode(get_password(&body.password) + &salt);

        let new_user = NewUser::new(body, salt);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();

        let mut response = Response::new();
        response.headers_mut().set(ContentType::json());

        match new_user.insert(&pg_pool, redis_pool) {
            Ok(cookies) => {
                let res = json!({
                    "status": true,
                });

                response.write_body(serde_json::to_string(&res).unwrap());

                let _ = set_cookie(&mut response, "blog_session".to_string(), cookies,
                                   None, Some("/".to_string()), None, Some(24));
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

    fn after(&self, _req: &Request, _res: &mut Response) -> SapperResult<()> {
        Ok(())
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        // http get /article/view_all limit==5 offset==0
        router.get("/article/view_all", Visitor::list_all_article);

        router.get("/article/view_all/:tag_id", Visitor::list_all_article_filter_by_tag);

        router.get("/article/view_comment/:id", Visitor::list_comments);

        // http get /article/view id==4
        router.get("/article/view", Visitor::view_article);

        // http post :8888/user/login account=admin password=admin
        router.post("/user/login", Visitor::login);

        // http post :8888/user/new account="k1234" password="1234" nickname="漂流" email="441594700@qq.com" say=""
        router.post("/user/new", Visitor::create_user);

        Ok(())
    }
}
