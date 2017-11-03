use sapper::{ SapperModule, SapperRouter, Response, Request, Result };
use sapper_std::{ PathParams, QueryParams, JsonParams };
use serde_json;
use diesel;
use diesel::{ FilterDsl, ExpressionMethods, ExecuteDsl, LoadDsl, SelectDsl, OrderDsl, LimitDsl };

use super::posts::dsl::{ posts, id, title, published, create_time, modify_time, content };
use { Posts, NewPost, ArticleList, establish_connection };

pub struct Article;

impl Article {
    fn create_article(req: &mut Request) -> Result<Response> {
        let body: NewPost = get_json_params!(req);
        let conn = establish_connection();

        if NewPost::new(body.title, body.content).insert(&conn) {
            res_json!(json!({"status": true}))
        } else {
            res_json!(json!({"status": false}))
        }
    }

    fn delete_article(req: &mut Request) -> Result<Response> {
        let params = get_path_params!(req);
        let article_id: i32 = t_param!(params, "id").clone().parse().unwrap();
        let conn = establish_connection();

        let num_deleted = diesel::delete(posts.filter(id.eq(article_id)))
            .execute(&conn)
            .expect("Error deleting posts");

        res_json!(json!({"status": true, "num_deleted": num_deleted }))
    }

    fn admin_view_article(req: &mut Request) -> Result<Response> {
        let params = get_query_params!(req);
        let article_id = t_param_parse!(params, "id", i32);
        let conn = establish_connection();

        match posts.filter(id.eq(article_id)).load::<Posts>(&conn) {
            Ok(data) => {
                let res = json!({
                    "status": true,
                    "data": data
                });
                res_json!(res)
            }
            Err(_) => {
                let res = json!({
                    "status": false,
                    "error": format!("no this article, id: {}", article_id)
                });
                res_json!(res)
            }
        }
    }

    fn view_article(req: &mut Request) -> Result<Response> {
        let params = get_query_params!(req);
        let article_id = t_param_parse!(params, "id", i32);
        let conn = establish_connection();

        match posts.filter(id.eq(article_id)).filter(published.eq(true)).load::<Posts>(&conn) {
            Ok(data) => {
                let res = json!({
                    "status": true,
                    "data": data
                });
                res_json!(res)
            }
            Err(_) => {
                let res = json!({
                    "status": false,
                    "error": format!("no this article, id: {}", article_id)
                });
                res_json!(res)
            }
        }
    }

    fn admin_list_all_article(req: &mut Request) -> Result<Response> {
        let params = get_query_params!(req);
        let limit = t_param_parse!(params, "limit", i64);
        let conn = establish_connection();
        match posts
            .select((id, title, published, create_time, modify_time))
            .order(create_time)
            .limit(limit)
            .load::<ArticleList>(&conn) {
            Ok(data) => {
                let res = json!({
                    "status": true,
                    "data": data
                });
                res_json!(res)
            }
            Err(err) => {
                let res = json!({
                    "status": false,
                    "error": format!("{}", err)
                });
                res_json!(res)
            }
        }
    }

    fn list_all_article(req: &mut Request) -> Result<Response> {
        let params = get_query_params!(req);
        let limit = t_param_parse!(params, "limit", i64);
        let conn = establish_connection();
        match posts
            .select((id, title, published, create_time, modify_time))
            .filter(published.eq(true))
            .order(create_time)
            .limit(limit)
            .load::<ArticleList>(&conn) {
            Ok(data) => {
                let res = json!({
                    "status": true,
                    "data": data
                });
                res_json!(res)
            }
            Err(err) => {
                let res = json!({
                    "status": false,
                    "error": format!("{}", err)
                });
                res_json!(res)
            }
        }
    }

    fn edit_article(req: &mut Request) -> Result<Response> {
        #[derive(Deserialize, Serialize)]
        struct AditArticle {
            id: i32,
            title: String,
            content: String
        }

        let body: AditArticle = get_json_params!(req);

        let conn = establish_connection();

        match diesel::update(posts.filter(id.eq(body.id)))
            .set((title.eq(body.title), content.eq(body.content)))
            .execute(&conn) {
            Ok(num_update) => {
                let res = json!({
                    "status": true,
                    "num_update": num_update
                });
                res_json!(res)
            }
            Err(err) => {
                let res = json!({
                    "status": false,
                    "error": format!("{}", err)
                });
                res_json!(res)
            }
        }
    }

    fn update_publish(req: &mut Request) -> Result<Response> {
        #[derive(Deserialize, Serialize)]
        struct ModifyPublish {
            id: i32,
            publish: bool
        }

        let body: ModifyPublish = get_json_params!(req);

        let conn = establish_connection();

        match diesel::update(posts.filter(id.eq(body.id)))
            .set(published.eq(body.publish))
            .execute(&conn) {
            Ok(num_update) => {
                let res = json!({
                    "status": true,
                    "num_update": num_update
                });
                res_json!(res)
            }
            Err(err) => {
                let res = json!({
                    "status": false,
                    "error": format!("{}", err)
                });
                res_json!(res)
            }
        }
    }
}

impl SapperModule for Article {
    fn before(&self, _req: &mut Request) -> Result<()> {
        Ok(())
    }

    fn after(&self, _req: &Request, _res: &mut Response) -> Result<()> {
        Ok(())
    }

    fn router(&self, router: &mut SapperRouter) -> Result<()> {
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
