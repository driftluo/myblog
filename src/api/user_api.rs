use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use sapper_std::{ JsonParams, SessionVal };
use serde_json;

use super::super::{ Postgresql, UserInfo, ChangePassword, Redis, user_verification_cookie,
                    admin_verification_cookie, AdminSession, LoginUser };

pub struct User;

impl User {
    fn view_user(req: &mut Request) -> SapperResult<Response> {
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let admin = req.ext().get::<AdminSession>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let res = match UserInfo::view_user_with_cookie(&pg_pool, redis_pool, cookie, admin) {
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

    fn sign_out(req: &mut Request) -> SapperResult<Response> {
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let admin = req.ext().get::<AdminSession>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let res = json!({"status": LoginUser::sign_out(redis_pool, cookie, admin) });
        res_json!(res)
    }
}

impl SapperModule for User {
    #[allow(unused_assignments)]
    fn before(&self, req: &mut Request) -> SapperResult<Option<Response>> {
        let mut admin_status = false;
        let mut user_status = false;
        {
            let cookie = req.ext().get::<SessionVal>();
            let redis_pool = req.ext().get::<Redis>().unwrap();
            admin_status = admin_verification_cookie(cookie, redis_pool);
            user_status = user_verification_cookie(cookie, redis_pool);
        }
        req.ext_mut().insert::<AdminSession>(admin_status);

        match user_status {
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

        // http get :8888/user/view
        router.get("/user/view", User::view_user);

        router.get("/user/sign_out", User::sign_out);

        Ok(())
    }
}
