use salvo::{
    http::{StatusCode, StatusError},
    prelude::{async_trait, fn_handler},
    Request, Response, Router,
};

use crate::{
    api::{block_no_admin, JsonErrResponse, JsonOkResponse},
    models::user::{ChangePermission, DisabledUser, UserInfo},
    utils::{from_code, parse_json_body, parse_last_path, parse_query, set_json_response},
    Routers,
};

#[fn_handler]
async fn delete_user(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let id = parse_last_path::<uuid::Uuid>(req)?;

    match UserInfo::delete(id).await {
        Ok(num) => set_json_response(res, 32, &JsonOkResponse::ok(num)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }

    Ok(())
}

#[fn_handler]
async fn view_user_list(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let limit = parse_query::<i64>(req, "limit")?;
    let offset = parse_query::<i64>(req, "offset")?;

    match UserInfo::view_user_list(limit, offset).await {
        Ok(data) => set_json_response(res, 128, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }

    Ok(())
}

#[fn_handler]
async fn change_permission(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let body = parse_json_body::<ChangePermission>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    match UserInfo::change_permission(body).await {
        Ok(data) => set_json_response(res, 32, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
    Ok(())
}

#[fn_handler]
async fn change_disabled(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let body = parse_json_body::<DisabledUser>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    match UserInfo::disabled_user(body).await {
        Ok(data) => set_json_response(res, 32, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
    Ok(())
}

pub struct AdminUser;
impl Routers for AdminUser {
    fn build(self) -> Vec<Router> {
        use crate::api::PREFIX;
        vec![Router::new()
            .path(PREFIX.to_owned() + "user")
            .hoop(block_no_admin)
            // http get {ip}/user/view_all limit==5 offset==0
            .push(Router::new().path("view_all").get(view_user_list))
            // http post {ip}/user/delete/uuid
            .push(Router::new().path("delete/<id>").post(delete_user))
            // http post {ip}/user/permission id:=uuid permission:=0
            .push(Router::new().path("permission").post(change_permission))
            // http post {ip}/user/permission id:=uuid disabled:=1
            .push(Router::new().path("delete/disable").post(change_disabled))]
    }
}
