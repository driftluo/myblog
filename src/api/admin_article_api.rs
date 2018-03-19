use sapper::{Error as SapperError, Request, Response, Result as SapperResult, SapperModule,
             SapperRouter};
use sapper_std::{JsonParams, PathParams, QueryParams, SessionVal};
use serde_json;
use uuid::Uuid;

use super::super::{admin_verification_cookie, ArticleList, ArticlesWithTag, EditArticle,
                   ModifyPublish, NewArticle, Postgresql, Redis};

pub struct AdminArticle;

impl AdminArticle {
    fn create_article(req: &mut Request) -> SapperResult<Response> {
        let body: NewArticle = get_json_params!(req);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        if body.insert(&pg_pool) {
            res_json!(json!({"status": true}))
        } else {
            res_json!(json!({"status": false}))
        }
    }

    fn delete_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_path_params!(req);
        let article_id: Uuid = t_param!(params, "id").clone().parse().unwrap();
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let res = match ArticlesWithTag::delete_with_id(&pg_pool, article_id) {
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

    fn admin_view_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let article_id = t_param_parse!(params, "id", Uuid);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let res = match ArticlesWithTag::query_article(&pg_pool, article_id, true) {
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

    fn admin_view_raw_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let article_id = t_param_parse!(params, "id", Uuid);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let res = match ArticlesWithTag::query_raw_article(&pg_pool, article_id) {
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

    fn admin_list_all_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let limit = t_param_parse!(params, "limit", i64);
        let offset = t_param_parse!(params, "offset", i64);
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match ArticleList::query_list_article(&pg_pool, limit, offset, true) {
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

    fn edit_article(req: &mut Request) -> SapperResult<Response> {
        let body: EditArticle = get_json_params!(req);

        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let res = match body.edit_article(&pg_pool) {
            Ok(num_update) => json!({
                    "status": true,
                    "num_update": num_update
                }),
            Err(err) => json!({
                    "status": false,
                    "error": format!("{}", err)
                }),
        };
        res_json!(res)
    }

    fn update_publish(req: &mut Request) -> SapperResult<Response> {
        let body: ModifyPublish = get_json_params!(req);

        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();

        let res = match ArticlesWithTag::publish_article(&pg_pool, body) {
            Ok(num_update) => json!({
                    "status": true,
                    "num_update": num_update
                }),
            Err(err) => json!({
                    "status": false,
                    "error": format!("{}", err)
                }),
        };
        res_json!(res)
    }
}

impl SapperModule for AdminArticle {
    fn before(&self, req: &mut Request) -> SapperResult<()> {
        let cookie = req.ext().get::<SessionVal>();
        let redis_pool = req.ext().get::<Redis>().unwrap();
        match admin_verification_cookie(cookie, redis_pool) {
            true => Ok(()),
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
        // http get /article/admin/view id==4
        router.get("/article/admin/view", AdminArticle::admin_view_article);

        router.get(
            "/article/admin/view_raw",
            AdminArticle::admin_view_raw_article,
        );

        // http get /article/admin/view_all limit==5 offset==0
        router.get(
            "/article/admin/view_all",
            AdminArticle::admin_list_all_article,
        );

        // http post /article/new title=something raw_content=something
        router.post("/article/new", AdminArticle::create_article);

        // http post /article/delete/3
        router.post("/article/delete/:id", AdminArticle::delete_article);

        // http post /article/edit id:=1 title=something raw_content=something
        router.post("/article/edit", AdminArticle::edit_article);

        // http post /article/publish id:=5 published:=true
        router.post("/article/publish", AdminArticle::update_publish);

        Ok(())
    }
}
