use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use serde_json;
use sapper_std::{ JsonParams, QueryParams, PathParams };

use { random_string, sha3_256_encode, establish_connection,
      UserInfo, Users, NewUser, ChangePassword, RegisteredUser, EditUser };

pub struct User;

impl User {
    fn create_user(req: &mut Request) -> SapperResult<Response> {
        let mut body: RegisteredUser = get_json_params!(req);
        let salt = random_string(6);
        body.password = sha3_256_encode(body.password + &salt);

        let new_user = NewUser::new(body, salt);
        let conn = establish_connection();

        if new_user.insert(&conn) {
            res_json!(json!({"status": true}))
        } else {
            res_json!(json!({"status": true}))
        }
    }

    fn change_pwd(req: &mut Request) -> SapperResult<Response> {
        let body: ChangePassword = get_json_params!(req);
        let conn = establish_connection();
        let res = if !body.verification(&conn) {
            json!({
                "status": false,
                "error": format!("fno this user, id: {}", body.id)
            })
        } else {
            match body.change_password(&conn) {
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
            }
        };
        res_json!(res)
    }

    fn delete_user(req: &mut Request) -> SapperResult<Response> {
        let params = get_path_params!(req);
        let user_id: i32 = t_param!(params, "id").clone().parse().unwrap();
        let conn = establish_connection();

        let res = match Users::delete(&conn, user_id) {
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

    fn view_user(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let user_id = t_param_parse!(params, "id", i32);
        let conn = establish_connection();

        let res = match UserInfo::view_user(&conn, user_id) {
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

    fn edit_user(req: &mut Request) -> SapperResult<Response> {
        let body: EditUser = get_json_params!(req);
        let conn = establish_connection();

        let res = match Users::edit_user(&conn, body) {
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

impl SapperModule for User {
    fn before(&self, _req: &mut Request) -> SapperResult<()> {
        Ok(())
    }

    fn after(&self, _req: &Request, _res: &mut Response) -> SapperResult<()> {
        Ok(())
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        // http :8888/user/view id==1
        router.get("/user/view", User::view_user);
        router.post("/user/new", User::create_user);
        router.post("/user/change_pwd", User::change_pwd);
        router.post("/user/delete/:id", User::delete_user);
        router.post("/user/edit", User::edit_user);
        Ok(())
    }
}
