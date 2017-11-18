use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use sapper_std::{ QueryParams, JsonParams, set_cookie };
use sapper::header::ContentType;
use serde_json;

use super::super::{ Articles, RegisteredUser, NewUser, sha3_256_encode, random_string,
                    ArticleList, Postgresql, Redis, LoginUser };

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

    fn view_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let article_id = t_param_parse!(params, "id", i32);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let res = match Articles::query_article(&pg_pool, article_id, false) {
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
        match body.verification(&pg_pool, redis_pool) {
            Ok(cookies) => {
                let res = json!({
                    "status": true,
                });

                response.write_body(serde_json::to_string(&res).unwrap());
                let _ = set_cookie(&mut response, "blog_session".to_string(), cookies,
                                   None, None, None, None);
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
        body.password = sha3_256_encode(body.password + &salt);

        let new_user = NewUser::new(body, salt);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        if new_user.insert(&pg_pool) {
            res_json!(json!({"status": true}))
        } else {
            res_json!(json!({"status": false}))
        }
    }
}

impl SapperModule for Visitor {
    fn before(&self, _req: &mut Request) -> SapperResult<Option<Response>> {
        Ok(None)
    }

    fn after(&self, _req: &Request, _res: &mut Response) -> SapperResult<()> {
        Ok(())
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        // http get /article/view_all limit==5 offset==0
        router.get("/article/view_all", Visitor::list_all_article);

        // http get /article/view id==4
        router.get("/article/view", Visitor::view_article);

        // http post :8888/user/login account=admin password=admin
        router.post("/user/login", Visitor::login);

        // http post :8888/user/new account="k1234" password="1234" nickname="漂流" email="441594700@qq.com" say=""
        router.post("/user/new", Visitor::create_user);

        Ok(())
    }
}
