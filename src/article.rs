use sapper::{ SapperModule, SapperRouter, Response, Request, Result };
use sapper_std::{ PathParams, QueryParams};
use serde_json;
use diesel;
use diesel::{ FilterDsl, ExpressionMethods, ExecuteDsl, LoadDsl };

use super::posts::dsl::{ posts, id, published };
use { Posts, NewPost, establish_connection };

pub struct Article;

impl Article {
    fn new_article(req: &mut Request) -> Result<Response> {
        let post: NewPost= serde_json::from_slice(req.body().unwrap().as_slice()).unwrap();
        let conn = establish_connection();

        if NewPost::new(post.title, post.content).insert(&conn) {
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

        res_json!(json!({"status": true, "num_deleted": num_deleted}))
    }

    fn view_article(req: &mut Request) -> Result<Response> {
        let params = get_query_params!(req);
        let article_id = t_param_parse!(params, "id", i32);
        let conn = establish_connection();

        match posts.filter(published.eq(true)).filter(id.eq(article_id)).load::<Posts>(&conn) {
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
}

impl SapperModule for Article {
    fn before(&self, _req: &mut Request) -> Result<()> {
        Ok(())
    }

    fn after(&self, _req: &Request, _res: &mut Response) -> Result<()> {
        Ok(())
    }

    fn router(&self, router: &mut SapperRouter) -> Result<()> {
        router.get("/article/view", Article::view_article);
        router.post("/article/new", Article::new_article);
        router.post("/article/delete/:id", Article::delete_article);
        Ok(())
    }
}
