use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult, Error as SapperError };
use sapper_std::{ JsonParams, SessionVal };
use serde_json;

use super::super::{ Postgresql, UserInfo, ChangePassword, Redis, user_verification_cookie,
                    admin_verification_cookie, AdminSession, LoginUser, EditUser, NewComments };

pub struct User;

impl User {
    fn view_user(req: &mut Request) -> SapperResult<Response> {
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let admin = req.ext().get::<AdminSession>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let mut res = json!({
                    "status": true,
                });
        res["data"] = serde_json::from_str(&UserInfo::view_user_with_cookie(redis_pool, cookie, admin)).unwrap();
        res_json!(res)
    }

    fn change_pwd(req: &mut Request) -> SapperResult<Response> {
        let body: ChangePassword = get_json_params!(req);
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let admin = req.ext().get::<AdminSession>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match body.change_password(&pg_pool, redis_pool, cookie, admin) {
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

    fn edit(req: &mut Request) -> SapperResult<Response> {
        let body: EditUser = get_json_params!(req);
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let admin = req.ext().get::<AdminSession>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match body.edit_user(&pg_pool, redis_pool, cookie, admin) {
            Ok(num_edit) => json!({
                "status": true,
                "num_edit": num_edit
            }),
            Err(err) => json!({
                "status": false,
                "error": err
            })
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

    fn new_comment(req: &mut Request) -> SapperResult<Response> {
        let body: NewComments = get_json_params!(req);
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let admin = req.ext().get::<AdminSession>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match body.insert(&pg_pool, redis_pool, cookie, admin) {
            true => json!({
                "status": true
            }),
            false => json!({
                "status": false
            })
        };
        res_json!(res)
    }
}

impl SapperModule for User {
    #[allow(unused_assignments)]
    fn before(&self, req: &mut Request) -> SapperResult<()> {
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
            true => { Ok(()) }
            false => {
                let res = json!({
                    "status": false,
                    "error": String::from("Verification error")
                });
                Err(SapperError::CustomJson(res.to_string()))
            }
        }
    }

    fn after(&self, _req: &Request, _res: &mut Response) -> SapperResult<()> {
        Ok(())
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        // http post :8888/user/change_pwd old_password=1234 new_password=12345
        router.post("/user/change_pwd", User::change_pwd);

        // http get :8888/user/view
        router.get("/user/view", User::view_user);

        router.get("/user/sign_out", User::sign_out);

        router.post("/user/edit", User::edit);

        router.post("/comment/new", User::new_comment);

        Ok(())
    }
}
