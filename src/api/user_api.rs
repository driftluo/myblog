use sapper::{Error as SapperError, Request, Response, Result as SapperResult, SapperModule,
             SapperRouter};
use sapper_std::{JsonParams, SessionVal};
use serde_json;

use super::super::{ChangePassword, DeleteComment, EditUser, LoginUser, NewComments, Permissions,
                   Postgresql, Redis, UserInfo, UserNotify, ArticlesWithTag};

pub struct User;

impl User {
    fn view_user(req: &mut Request) -> SapperResult<Response> {
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let mut res = json!({
                    "status": true,
                });
        res["data"] =
            serde_json::from_str(&UserInfo::view_user_with_cookie(redis_pool, cookie)).unwrap();
        res_json!(res)
    }

    fn change_pwd(req: &mut Request) -> SapperResult<Response> {
        let body: ChangePassword = get_json_params!(req);
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match body.change_password(&pg_pool, redis_pool, cookie) {
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

    fn edit(req: &mut Request) -> SapperResult<Response> {
        let body: EditUser = get_json_params!(req);
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match body.edit_user(&pg_pool, redis_pool, cookie) {
            Ok(num_edit) => json!({
                "status": true,
                "num_edit": num_edit
            }),
            Err(err) => json!({
                "status": false,
                "error": err
            }),
        };
        res_json!(res)
    }

    fn sign_out(req: &mut Request) -> SapperResult<Response> {
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let res = json!({ "status": LoginUser::sign_out(redis_pool, cookie) });
        res_json!(res)
    }

    fn new_comment(req: &mut Request) -> SapperResult<Response> {
        let mut body: NewComments = get_json_params!(req);
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let user =
            serde_json::from_str::<UserInfo>(&UserInfo::view_user_with_cookie(redis_pool, cookie)).unwrap();
        let admin = UserInfo::view_admin(&pg_pool);
        let article = ArticlesWithTag::query_without_article(&pg_pool, body.article_id(), false).unwrap();

        if let Some(reply_user_id) = body.reply_user_id() {
            if user.id != reply_user_id {
                let user_reply_notify = UserNotify {
                    user_id: reply_user_id,
                    send_user_name: user.nickname.clone(),
                    article_id: article.id,
                    article_title: article.title.clone(),
                    notify_type: "reply".into(),
                };
                user_reply_notify.cache(&redis_pool);
            }
        }

        if user.groups != 0 {
            let comment_notify = UserNotify {
                user_id: admin.id,
                send_user_name: user.nickname.clone(),
                article_id: article.id,
                article_title: article.title.clone(),
                notify_type: "comment".into(),
            };
            comment_notify.cache(&redis_pool);
        }

        let res = json!({
                "status": body.insert(&pg_pool, redis_pool, cookie)
        });
        res_json!(res)
    }

    fn delete_comment(req: &mut Request) -> SapperResult<Response> {
        let body: DeleteComment = get_json_params!(req);
        let permission = req.ext().get::<Permissions>().unwrap();
        let cookie = req.ext().get::<SessionVal>().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let res = json!({
                "status": body.delete(&pg_pool, redis_pool, cookie, permission)
            });
        res_json!(res)
    }
}

impl SapperModule for User {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        let permission = req.ext().get::<Permissions>().unwrap();
        match *permission {
            Some(_) => Ok(()),
            None => {
                let res = json!({
                    "status": false,
                    "error": String::from("Verification error")
                });
                Err(SapperError::CustomJson(res.to_string()))
            }
        }
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        // http post :8888/user/change_pwd old_password=1234 new_password=12345
        router.post("/user/change_pwd", User::change_pwd);

        // http get :8888/user/view
        router.get("/user/view", User::view_user);

        router.get("/user/sign_out", User::sign_out);

        router.post("/user/edit", User::edit);

        router.post("/comment/new", User::new_comment);

        router.post("/comment/delete", User::delete_comment);

        Ok(())
    }
}
