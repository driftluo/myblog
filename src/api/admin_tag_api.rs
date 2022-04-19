use salvo::{
    http::{StatusCode, StatusError},
    prelude::{async_trait, fn_handler},
    Request, Response, Router,
};

use crate::{
    api::block_no_admin,
    api::{JsonErrResponse, JsonOkResponse},
    models::tag::{TagCount, Tags},
    utils::{from_code, parse_json_body, parse_last_path, parse_query, set_json_response},
    Routers,
};

#[fn_handler]
async fn create_tag(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct NewTag {
        tag: String,
    }
    let body = parse_json_body::<NewTag>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    match Tags::insert(&body.tag).await {
        Ok(data) => set_json_response(res, 32, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
    Ok(())
}

#[fn_handler]
async fn delete_tag(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let id = parse_last_path::<uuid::Uuid>(req)?;

    match Tags::delete_tag(id).await {
        Ok(data) => set_json_response(res, 32, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
    Ok(())
}

#[fn_handler]
async fn view_tag(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let limit = parse_query::<i64>(req, "limit")?;
    let offset = parse_query::<i64>(req, "offset")?;

    match TagCount::view_all_tag_count(limit, offset).await {
        Ok(data) => set_json_response(res, 128, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
    Ok(())
}

#[fn_handler]
async fn edit_tag(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let body = parse_json_body::<Tags>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    match body.edit_tag().await {
        Ok(data) => set_json_response(res, 32, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
    Ok(())
}

pub struct Tag;

impl Routers for Tag {
    fn build(self) -> Vec<Router> {
        use crate::api::PREFIX;
        vec![Router::new()
            .path(PREFIX.to_owned() + "tag")
            .hoop(block_no_admin)
            // http get {ip}/tag/view limit==5 offset==0
            .push(Router::new().path("view").get(view_tag))
            // http post {ip}/tag/new tag="Rust"
            .push(Router::new().path("new").post(create_tag))
            // http post {ip}/tag/delete/3
            .push(Router::new().path("delete/<id>").post(delete_tag))
            // http post :8888/tag/edit id:=2 tag="Linux&&Rust"
            .push(Router::new().path("edit").post(edit_tag))]
    }
}
