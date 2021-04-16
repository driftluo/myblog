use salvo::{
    http::{HttpError, StatusCode},
    prelude::{async_trait, fn_handler},
    Depot, Request, Response, Router, Writer,
};

use crate::{
    api::{block_unlogin, JsonErrResponse, JsonOkResponse},
    models::{
        articles::ArticlesWithTag,
        comment::{DeleteComment, NewComments},
        notify::UserNotify,
        user::{ChangePassword, EditUser, LoginUser, UserInfo},
    },
    utils::{from_code, parse_json_body, set_json_response},
    Routers, COOKIE, PERMISSION, USER_INFO,
};

#[fn_handler]
async fn view_user(depot: &mut Depot, res: &mut Response) {
    let info = depot.take::<_, UserInfo>(USER_INFO);
    set_json_response(res, 128, &JsonOkResponse::ok(info))
}

#[fn_handler]
async fn change_pwd(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), HttpError> {
    let body = parse_json_body::<ChangePassword>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    let cookie = depot.take::<_, String>(COOKIE);

    match body.change_password(&cookie).await {
        Ok(num) => set_json_response(res, 32, &JsonOkResponse::ok(num)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
    Ok(())
}

#[fn_handler]
async fn edit(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), HttpError> {
    let body = parse_json_body::<EditUser>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    let cookie = depot.take::<_, String>(COOKIE);

    match body.edit_user(&cookie).await {
        Ok(num) => set_json_response(res, 32, &JsonOkResponse::ok(num)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }

    Ok(())
}

#[fn_handler]
async fn sign_out(depot: &mut Depot, res: &mut Response) {
    let cookie = depot.take::<_, String>(COOKIE);
    let a = LoginUser::sign_out(&cookie).await;
    set_json_response(res, 32, &JsonOkResponse::status(a));
}

#[fn_handler]
async fn new_comment(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), HttpError> {
    let mut body = parse_json_body::<NewComments>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    let article = ArticlesWithTag::query_without_article(body.article_id(), false)
        .await
        .map_err(|_| from_code(StatusCode::NOT_FOUND, "Article doesn't exist"))?;
    let admin = UserInfo::view_admin().await;
    let user = depot.take::<_, UserInfo>(USER_INFO);

    match body.reply_user_id() {
        // Reply comment
        Some(reply_user_id) => {
            // Notification reply
            let user_reply_notify = UserNotify {
                user_id: reply_user_id,
                send_user_name: user.nickname.clone(),
                article_id: article.id,
                article_title: article.title.clone(),
                notify_type: "reply".into(),
            };
            user_reply_notify.cache().await;

            // If the sender is not an admin and also the responder is also not admin, notify admin
            if reply_user_id != admin.id && user.groups != 0 {
                let comment_notify = UserNotify {
                    user_id: admin.id,
                    send_user_name: user.nickname,
                    article_id: article.id,
                    article_title: article.title,
                    notify_type: "comment".into(),
                };
                comment_notify.cache().await;
            }
        }
        // Normal comment
        None => {
            if user.groups != 0 {
                let comment_notify = UserNotify {
                    user_id: admin.id,
                    send_user_name: user.nickname,
                    article_id: article.id,
                    article_title: article.title,
                    notify_type: "comment".into(),
                };
                comment_notify.cache().await;
            }
        }
    }

    set_json_response(res, 32, &JsonOkResponse::status(body.insert(user.id).await));
    Ok(())
}

#[fn_handler]
async fn delete_comment(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), HttpError> {
    let body = parse_json_body::<DeleteComment>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;
    let permission = depot.take::<_, Option<i16>>(PERMISSION);
    let info = depot.take::<_, UserInfo>(USER_INFO);

    set_json_response(
        res,
        32,
        &JsonOkResponse::status(body.delete(info.id, permission).await),
    );
    Ok(())
}

pub struct User;

impl Routers for User {
    fn build(self) -> Vec<Router> {
        use crate::api::PREFIX;
        vec![
            Router::new()
                .path(PREFIX.to_owned() + "user")
                .before(block_unlogin)
                // http {ip}/PREFIX/user/view
                .push(Router::new().path("view").get(view_user))
                // http post {ip}/user/change_pwd old_password=1234 new_password=12345
                .push(Router::new().path("change_pwd").post(change_pwd))
                // http {ip}/user/sign_out
                .push(Router::new().path("sign_out").get(sign_out))
                // http post {ip}/user/edit nickname=xxx say=xxx email=xxx
                .push(Router::new().path("edit").post(edit)),
            Router::new()
                .path(PREFIX.to_owned() + "comment")
                .before(block_unlogin)
                // http post {ip}/comment/new comment=xxx article_id=xxx reply_user_id=xxx
                .push(Router::new().path("new").post(new_comment))
                // http post {ip}/comment/delete comment_id=xxx user_id=xxx
                .push(Router::new().path("delete").post(delete_comment)),
        ]
    }
}
