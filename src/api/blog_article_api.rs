use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use sapper_std::{ PathParams, QueryParams, JsonParams };
use serde_json;

use super::super::{ EditArticle, ModifyPublish, Articles,
                    NewArticle, ArticleList, establish_connection };

pub struct Article;

impl Article {
    fn create_article(req: &mut Request) -> SapperResult<Response> {
        let body: NewArticle = get_json_params!(req);
        let conn = establish_connection();

        if body.insert(&conn) {
            res_json!(json!({"status": true}))
        } else {
            res_json!(json!({"status": false}))
        }
    }

    fn delete_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_path_params!(req);
        let article_id: i32 = t_param!(params, "id").clone().parse().unwrap();
        let conn = establish_connection();

        let res = match Articles::delete_with_id(&conn, article_id) {
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

    fn admin_view_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let article_id = t_param_parse!(params, "id", i32);
        let conn = establish_connection();

        let res = match Articles::query_article(&conn, article_id, true) {
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

    fn view_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let article_id = t_param_parse!(params, "id", i32);
        let conn = establish_connection();

        let res = match Articles::query_article(&conn, article_id, false) {
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

    fn admin_list_all_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let limit = t_param_parse!(params, "limit", i64);
        let offset = t_param_parse!(params, "offset", i64);
        let conn = establish_connection();
        let res = match ArticleList::query_list_article(&conn, limit, offset, true) {
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

    fn list_all_article(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let limit = t_param_parse!(params, "limit", i64);
        let offset = t_param_parse!(params, "offset", i64);
        let conn = establish_connection();
        let res = match ArticleList::query_list_article(&conn, limit, offset, false) {
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

    fn edit_article(req: &mut Request) -> SapperResult<Response> {

        let body: EditArticle = get_json_params!(req);

        let conn = establish_connection();

        let res = match Articles::edit_article(&conn, body) {
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

    fn update_publish(req: &mut Request) -> SapperResult<Response> {

        let body: ModifyPublish = get_json_params!(req);

        let conn = establish_connection();

        let res = match Articles::publish_article(&conn, body) {
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

impl SapperModule for Article {
    fn before(&self, _req: &mut Request) -> SapperResult<()> {
        Ok(())
    }

    fn after(&self, _req: &Request, _res: &mut Response) -> SapperResult<()> {
        Ok(())
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        // http get /article/admin/view id==4
        router.get("/article/admin/view", Article::admin_view_article);

        // http get /article/admin/view_all limit==5
        router.get("/article/admin/view_all", Article::admin_list_all_article);

        // http get /article/view id==4
        router.get("/article/view", Article::view_article);

        // http get /article/view_all limit==5
        router.get("/article/view_all", Article::list_all_article);

        // http post /article/new title=something content=something
        router.post("/article/new", Article::create_article);

        // http post /article/publish id:=5 published:=true
        router.post("/article/publish", Article::update_publish);

        // http post /article/delete/3
        router.post("/article/delete/:id", Article::delete_article);

        // http post /article/edit id:=1 title=something content=something
        router.post("/article/edit", Article::edit_article);
        Ok(())
    }
}
