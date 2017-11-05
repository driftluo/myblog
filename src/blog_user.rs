use sapper::{ SapperModule, SapperRouter, Response, Request, Result };
use serde_json;
use sapper_std::{ JsonParams };

use { random_string, md5_encode, establish_connection,
      UserInfo, Users, NewUser, ChangePassword, RegisteredUser };

pub struct User;

impl User {
    fn create_user(req: &mut Request) -> Result<Response> {
        let mut body: RegisteredUser = get_json_params!(req);
        let salt = random_string(6);
        body.password = md5_encode(body.password + &salt);

        let new_user = NewUser::new(body, salt);
        let conn = establish_connection();

        if new_user.insert(&conn) {
            res_json!(json!({"status": true}))
        } else {
            res_json!(json!({"status": true}))
        }
    }

    fn change_pwd(req: &mut Request) -> Result<Response> {
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
}

impl SapperModule for User {
    fn before(&self, _req: &mut Request) -> Result<()> {
        Ok(())
    }

    fn after(&self, _req: &Request, _res: &mut Response) -> Result<()> {
        Ok(())
    }

    fn router(&self, router: &mut SapperRouter) -> Result<()> {
        router.post("/user/new", User::create_user);
        router.post("/user/change_pwd", User::change_pwd);
        Ok(())
    }
}
