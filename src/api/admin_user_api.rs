use sapper::{
    Error as SapperError, Request, Response, Result as SapperResult, SapperModule, SapperRouter,
};
use sapper_std::{JsonParams, PathParams, QueryParams};
use serde_json::{self, json};
use uuid::Uuid;

use super::super::{
    ChangePermission, DisabledUser, Permissions, Postgresql, Redis, UserInfo, Users,
};

pub struct AdminUser;

impl AdminUser {
    fn delete_user(req: &mut Request) -> SapperResult<Response> {
        let params = get_path_params!(req);
        let user_id: Uuid = t_param!(params, "id").parse().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();

        let res = match Users::delete(&pg_pool, redis_pool, user_id) {
            Ok(num_deleted) => json!({
            "status": true,
            "num_deleted": num_deleted
            }),
            Err(err) => json!({
            "status": false,
            "error": err
            }),
        };
        res_json!(res)
    }

    fn view_user_list(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let limit = t_param_parse!(params, "limit", i64);
        let offset = t_param_parse!(params, "offset", i64);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match UserInfo::view_user_list(&pg_pool, limit, offset) {
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

    fn change_permission(req: &mut Request) -> SapperResult<Response> {
        let body: ChangePermission = get_json_params!(req);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match Users::change_permission(&pg_pool, body) {
            Ok(num_update) => json!({
                "status": true,
                "num_update": num_update
            }),
            Err(err) => json!({
                "status": false,
                "error": err.to_string()
            }),
        };
        res_json!(res)
    }

    fn change_disabled(req: &mut Request) -> SapperResult<Response> {
        let body: DisabledUser = get_json_params!(req);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match Users::disabled_user(&pg_pool, body) {
            Ok(num_update) => json!({
                "status": true,
                "num_update": num_update
            }),
            Err(err) => json!({
                "status": false,
                "error": err.to_string()
            }),
        };
        res_json!(res)
    }
}

impl SapperModule for AdminUser {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        let permission = req.ext().get::<Permissions>().unwrap();
        match *permission {
            Some(0) => Ok(()),
            _ => {
                let res = json!({
                    "status": false,
                    "error": String::from("Verification error")
                });
                Err(SapperError::CustomJson(res.to_string()))
            }
        }
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        // http get /user/view_all limit==5 offset==0
        router.get("/user/view_all", AdminUser::view_user_list);

        // http post :8888/user/delete/uuid
        router.post("/user/delete/:id", AdminUser::delete_user);

        // http post :8888/user/permission id:=uuid permission:=0
        router.post("/user/permission", AdminUser::change_permission);

        // http post :8888/user/permission id:=uuid disabled:=1
        router.post("/user/disable", AdminUser::change_disabled);

        Ok(())
    }
}
