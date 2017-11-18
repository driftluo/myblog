use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use serde_json;
use sapper_std::{ JsonParams, PathParams, SessionVal };

use super::super::{ Users, EditUser, Postgresql, Redis, admin_verification_cookie };

pub struct AdminUser;

impl AdminUser {
    fn delete_user(req: &mut Request) -> SapperResult<Response> {
        let params = get_path_params!(req);
        let user_id: i32 = t_param!(params, "id").clone().parse().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let res = match Users::delete(&pg_pool, user_id) {
            Ok(num_deleted) => {
                json!({
                    "status": true,
                    "num_deleted": num_deleted
                    })
            },
            Err(err) => {
                json!({
                    "status": false,
                    "error": err
                    })
            }
        };
        res_json!(res)
    }

    fn edit_user(req: &mut Request) -> SapperResult<Response> {
        let body: EditUser = get_json_params!(req);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let res = match Users::edit_user(&pg_pool, body) {
            Ok(num_update) => {
                json!({
                    "status": true,
                    "num_update": num_update
                })
            }
            Err(err) => {
                json!({
                    "status": false,
                    "error": format!("{}", err)
                })
            }
        };
        res_json!(res)
    }
}

impl SapperModule for AdminUser {
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

        // http post :8888/user/delete/2
        router.post("/user/delete/:id", AdminUser::delete_user);

        // http post :8888/user/edit id:=1 nickname="漂流"
        // say="仍需共生命的慷慨与繁华相爱，即使岁月以刻薄与荒芜相欺。" email=441594700@qq.com
        router.post("/user/edit", AdminUser::edit_user);

        Ok(())
    }
}
