use chrono::offset::TimeZone;
use rss::{ChannelBuilder, Item, ItemBuilder};
use salvo::{
    http::{response::Body, StatusCode, StatusError},
    hyper::header::{self, HeaderValue},
    prelude::{async_trait, fn_handler},
    Depot, Request, Response, Router,
};
use uuid::Uuid;

use crate::{
    api::{current_size, JsonErrResponse, JsonOkResponse},
    models::{
        articles::{ArticleList, ArticlesWithTag},
        comment::Comments,
        user::{LoginUser, RegisteredUser, UserInfo},
    },
    utils::{
        from_code,
        github_information::{get_github_account_nickname_address, get_github_token},
        parse_json_body, parse_last_path, parse_query, set_cookie, set_json_response,
        set_plain_text_response,
    },
    web::Cache,
    Routers, PERMISSION, USER_INFO,
};
use bytes::BytesMut;

#[fn_handler]
async fn list_all_article(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let limit = parse_query::<i64>(req, "limit")?;
    let offset = parse_query::<i64>(req, "offset")?;

    if offset > current_size().await as i64 {
        return Err(from_code(StatusCode::BAD_REQUEST, "Query param invalid"));
    }

    match ArticleList::query_article(limit, offset, false).await {
        Ok(data) => set_json_response(res, 128, &JsonOkResponse::ok(data)),
        Err(err) => set_json_response(res, 32, &JsonErrResponse::err(err)),
    }
    Ok(())
}

#[fn_handler]
async fn list_all_article_filter_by_tag(
    req: &mut Request,
    res: &mut Response,
) -> Result<(), StatusError> {
    let tag_id = parse_last_path::<Uuid>(req)?;

    match ArticleList::query_with_tag(tag_id).await {
        Ok(data) => set_json_response(res, 128, &JsonOkResponse::ok(data)),
        Err(err) => set_json_response(res, 32, &JsonErrResponse::err(err)),
    }
    Ok(())
}

#[fn_handler]
async fn list_comments(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), StatusError> {
    let article_id = parse_last_path::<Uuid>(req)?;
    let limit = parse_query::<i64>(req, "limit")?;
    let offset = parse_query::<i64>(req, "offset")?;

    let (user_id, admin) = match depot.remove::<Option<i16>>(PERMISSION).unwrap() {
        Some(0) => {
            let info = depot.remove::<UserInfo>(USER_INFO).unwrap();
            (Some(info.id), true)
        }
        Some(_) => {
            let info = depot.remove::<UserInfo>(USER_INFO).unwrap();
            (Some(info.id), false)
        }
        None => (None, false),
    };

    match Comments::query(limit, offset, article_id).await {
        Ok(data) => {
            #[derive(serde::Deserialize, serde::Serialize)]
            struct Tmp<T> {
                status: bool,
                data: T,
                admin: bool,
                user_id: Option<Uuid>,
            }
            set_json_response(
                res,
                data.len() + 32,
                &Tmp {
                    status: true,
                    data,
                    admin,
                    user_id,
                },
            )
        }
        Err(err) => set_json_response(res, 32, &JsonErrResponse::err(err)),
    }

    Ok(())
}

#[fn_handler]
async fn view_article(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let id = parse_query::<Uuid>(req, "id")?;

    match ArticlesWithTag::query_without_article(id, false).await {
        Ok(data) => set_json_response(res, 128, &JsonOkResponse::ok(data)),
        Err(err) => set_json_response(res, 32, &JsonErrResponse::err(err)),
    }

    Ok(())
}

#[fn_handler]
async fn login(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let body = parse_json_body::<LoginUser>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    let max_age: Option<i64> = if body.get_remember() {
        Some(24 * 90)
    } else {
        None
    };

    match body.verification(&max_age).await {
        Ok(cookie) => {
            set_cookie(res, cookie, None, Some("/"), None, max_age);
            set_plain_text_response(res, BytesMut::from(r#"{"status": true}"#));
        }
        Err(err) => set_json_response(res, 32, &JsonErrResponse::err(err)),
    }

    Ok(())
}

#[fn_handler]
async fn login_with_github(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let code = parse_query::<String>(req, "code")?;

    let token = get_github_token(&code)
        .await
        .map_err(|e| from_code(StatusCode::NOT_ACCEPTABLE, e))?;
    let (account, nickname, github_address) = get_github_account_nickname_address(&token)
        .await
        .map_err(|e| from_code(StatusCode::NOT_ACCEPTABLE, e))?;

    match LoginUser::login_with_github(github_address, nickname, account, &token).await {
        Ok(cookie) => {
            set_cookie(res, cookie, None, Some("/"), None, Some(24));
            res.set_status_code(StatusCode::FOUND);
            res.headers_mut()
                .insert(header::LOCATION, "/home".parse().unwrap());
            set_plain_text_response(res, BytesMut::from(r#"{"status": true}"#));
        }
        Err(err) => set_json_response(res, 32, &JsonErrResponse::err(err)),
    }

    Ok(())
}

#[fn_handler]
async fn create_user(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let body = parse_json_body::<RegisteredUser>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    match body.insert().await {
        Ok(cookie) => {
            set_cookie(res, cookie, None, Some("/"), None, Some(24));
            set_plain_text_response(res, BytesMut::from(r#"{"status": true}"#));
        }
        Err(err) => set_json_response(res, 32, &JsonErrResponse::err(err)),
    }
    Ok(())
}

#[fn_handler]
async fn rss_path(res: &mut Response) {
    let mut channel = ChannelBuilder::default()
        .title("driftluo's blog")
        .link("https://driftluo.com")
        .description("This is driftluo's Personal Blog's RSS feed.")
        .build()
        .unwrap();

    let hour = 3600;
    let fix_offset = chrono::FixedOffset::east(8 * hour);

    match ArticleList::query_article(10, 0, false).await {
        Ok(articles) => {
            let mut items: Vec<Item> = Vec::with_capacity(10);
            for article in articles {
                let item = ItemBuilder::default()
                    .title(article.title.clone())
                    .link(
                        "https://www.driftluo.com".to_owned()
                            + "/article/"
                            + &article.id.to_string(),
                    )
                    .description(article.title)
                    .pub_date(
                        fix_offset
                            .from_local_datetime(&article.create_time)
                            .unwrap()
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string(),
                    )
                    .build()
                    .unwrap();

                items.push(item);
            }
            channel.set_items(items);
            let mut bytes = Cache::new();
            channel.write_to(&mut bytes).unwrap();
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/xml; charset=utf-8"),
            );
            res.set_body(Some(Body::Bytes(bytes.into_inner())));
            res.set_status_code(StatusCode::OK)
        }
        Err(err) => set_json_response(res, 32, &JsonErrResponse::err(err)),
    }
}

pub struct Visitor;

impl Routers for Visitor {
    fn build(self) -> Vec<Router> {
        use crate::api::PREFIX;
        vec![
            // http {ip}/PREFIX/article/view_all?limit={number}&&offset={number}
            Router::new()
                .path(PREFIX.to_owned() + "article/view_all")
                .get(list_all_article),
            // http {ip}/PREFIX/article/view_all/<tag_id>
            Router::new()
                .path(PREFIX.to_owned() + "article/view_all/<tag_id>")
                .get(list_all_article_filter_by_tag),
            // http {ip}/PREFIX/article/view_all/<tag_id>
            Router::new()
                .path(PREFIX.to_owned() + "article/view_comment/<id>")
                .get(list_comments),
            // http {ip}/PREFIX/article/view/<id>
            Router::new()
                .path(PREFIX.to_owned() + "article/view")
                .get(view_article),
            // http {ip}/PREFIX/login_with_github
            Router::new()
                .path(PREFIX.to_owned() + "login_with_github")
                .get(login_with_github),
            // http POST {ip}/PREFIX/user/login?code={code}
            Router::new()
                .path(PREFIX.to_owned() + "user/login")
                .post(login),
            // http POST {ip}/PREFIX/user/new account={} password={} remember:={bool}
            Router::new()
                .path(PREFIX.to_owned() + "user/new")
                .post(create_user),
            // http {ip}/rss
            Router::new().path("rss").get(rss_path),
        ]
    }
}
