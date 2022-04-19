use salvo::{
    http::{StatusCode, StatusError},
    prelude::{async_trait, fn_handler},
    Request, Response, Router,
};

use crate::{
    api::{block_no_admin, size_add, size_reduce, JsonErrResponse, JsonOkResponse},
    models::articles::{ArticleList, ArticlesWithTag, EditArticle, ModifyPublish, NewArticle},
    utils::{from_code, parse_json_body, parse_last_path, parse_query, set_json_response},
    Routers,
};

#[fn_handler]
async fn create_article(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let body = parse_json_body::<NewArticle>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    tokio::spawn(async { size_add().await });
    set_json_response(res, 32, &JsonOkResponse::status(body.insert().await));
    Ok(())
}

#[fn_handler]
async fn delete_article(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let id = parse_last_path::<uuid::Uuid>(req)?;

    match ArticlesWithTag::delete_with_id(id).await {
        Ok(data) => {
            tokio::spawn(async { size_reduce().await });
            set_json_response(res, 32, &JsonOkResponse::ok(data))
        }
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }

    Ok(())
}

#[fn_handler]
async fn admin_view_article(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let id = parse_query::<uuid::Uuid>(req, "id")?;

    match ArticlesWithTag::query_without_article(id, true).await {
        Ok(data) => set_json_response(res, 128, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
    Ok(())
}

#[fn_handler]
async fn admin_view_raw_article(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let id = parse_query::<uuid::Uuid>(req, "id")?;

    match ArticlesWithTag::query_raw_article(id).await {
        Ok(data) => set_json_response(res, 128, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
    Ok(())
}

#[fn_handler]
async fn admin_list_all_article(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let limit = parse_query::<i64>(req, "limit")?;
    let offset = parse_query::<i64>(req, "offset")?;

    match ArticleList::query_article(limit, offset, true).await {
        Ok(data) => set_json_response(res, 128, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
    Ok(())
}

#[fn_handler]
async fn admin_list_all_unpublished(
    req: &mut Request,
    res: &mut Response,
) -> Result<(), StatusError> {
    let limit = parse_query::<i64>(req, "limit")?;
    let offset = parse_query::<i64>(req, "offset")?;

    match ArticleList::view_unpublished(limit, offset).await {
        Ok(data) => set_json_response(res, 128, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
    Ok(())
}

#[fn_handler]
async fn edit_article(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let body = parse_json_body::<EditArticle>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    match body.edit_article().await {
        Ok(data) => set_json_response(res, 32, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
    Ok(())
}

#[fn_handler]
async fn update_publish(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let body = parse_json_body::<ModifyPublish>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    match ArticlesWithTag::publish_article(body).await {
        Ok(data) => set_json_response(res, 32, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
    Ok(())
}

#[fn_handler]
async fn upload(req: &mut Request, res: &mut Response) {
    match req.get_files("files").await {
        Some(files) => {
            let mut msgs = Vec::with_capacity(files.len());
            for file in files {
                let dest = match file.file_name() {
                    Some(ref name) => format!("static/images/{}", name),
                    None => format!("static/images/{}", uuid::Uuid::new_v4().to_hyphenated()),
                };
                match tokio::fs::copy(&file.path(), ::std::path::Path::new(&dest)).await {
                    Ok(_) => {
                        msgs.push(dest);
                    }
                    Err(e) => {
                        set_json_response(res, 32, &JsonErrResponse::err(e.to_string()));
                        return;
                    }
                }
            }
            set_json_response(res, 32, &JsonOkResponse::ok(msgs))
        }
        None => {
            set_json_response(res, 32, &JsonErrResponse::err("file not found in request"));
            res.set_status_code(StatusCode::BAD_REQUEST);
        }
    }
}

pub struct AdminArticle;

impl Routers for AdminArticle {
    fn build(self) -> Vec<Router> {
        use crate::api::PREFIX;
        vec![
            Router::new()
                .path(PREFIX.to_owned() + "article")
                .hoop(block_no_admin)
                // http get /article/admin/view?id==4
                .push(Router::new().path("admin/view").get(admin_view_article))
                // http get /article/admin/view_raw?id==4
                .push(
                    Router::new()
                        .path("admin/view_raw")
                        .get(admin_view_raw_article),
                )
                // http get /article/admin/view_all limit==5 offset==0
                .push(
                    Router::new()
                        .path("admin/view_all")
                        .get(admin_list_all_article),
                )
                // http get /article/admin/view_unpublished limit==5 offset==0
                .push(
                    Router::new()
                        .path("admin/view_unpublished")
                        .get(admin_list_all_unpublished),
                )
                // http post /article/new title=something raw_content=something
                .push(Router::new().path("new").post(create_article))
                // http post /article/delete/3
                .push(Router::new().path("delete/<id>").post(delete_article))
                // http post /article/edit id:=1 title=something raw_content=something
                .push(Router::new().path("edit").post(edit_article))
                // http post /article/publish id:=5 published:=true
                .push(Router::new().path("publish").post(update_publish)),
            // http post /upload
            Router::new()
                .path(PREFIX.to_owned() + "upload")
                .hoop(block_no_admin)
                .post(upload),
        ]
    }
}
