use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use sapper_std::{ QueryParams, JsonParams, SessionVal };
use serde_json;

use super::super::{ Postgresql, UserInfo, ChangePassword, Redis, user_verification_cookie };

pub struct User;

impl User {
    fn view_user(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let user_id = t_param_parse!(params, "id", i32);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let res = match UserInfo::view_user(&pg_pool, user_id) {
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

    fn change_pwd(req: &mut Request) -> SapperResult<Response> {
        let body: ChangePassword = get_json_params!(req);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = if !body.verification(&pg_pool) {
            json!({
                "status": false,
                "error": format!("no this user, id: {}", body.id)
            })
        } else {
            match body.change_password(&pg_pool) {
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
            }
        };
        res_json!(res)
    }
}

impl SapperModule for User {
    fn before(&self, req: &mut Request) -> SapperResult<Option<Response>> {
        let cookie = req.ext().get::<SessionVal>();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        match user_verification_cookie(cookie, redis_pool) {
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
        // http post :8888/user/change_pwd id:=1 old_password=1234 new_password=12345
        router.post("/user/change_pwd", User::change_pwd);

        // http get :8888/user/view id==1
        router.get("/user/view", User::view_user);

        Ok(())
    }
}
