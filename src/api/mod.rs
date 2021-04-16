mod admin_article_api;
mod admin_chart_data_api;
mod admin_tag_api;
mod admin_user_api;
mod user_api;
mod visitor_api;

pub use admin_article_api::AdminArticle;
pub use admin_chart_data_api::ChartData;
pub use admin_tag_api::Tag;
pub use admin_user_api::AdminUser;
pub use user_api::User;
pub use visitor_api::Visitor;

use salvo::prelude::{async_trait, fn_handler, Writer};

// todo: remove on delete all template
const PREFIX: &str = "api/v1/";

#[fn_handler]
async fn block_unlogin(depot: &mut salvo::Depot) -> Result<(), salvo::http::HttpError> {
    match depot.try_borrow::<_, Option<i16>>(crate::PERMISSION) {
        Some(Some(_)) => Ok(()),
        _ => Err(crate::utils::from_code(
            salvo::hyper::StatusCode::FORBIDDEN,
            "No permission",
        )),
    }
}

#[fn_handler]
async fn block_no_admin(depot: &mut salvo::Depot) -> Result<(), salvo::http::HttpError> {
    match depot.try_borrow::<_, Option<i16>>(crate::PERMISSION) {
        Some(Some(0)) => Ok(()),
        _ => Err(crate::utils::from_code(
            salvo::hyper::StatusCode::FORBIDDEN,
            "No permission",
        )),
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct JsonOkResponse<T> {
    status: bool,
    data: T,
}

impl<T> JsonOkResponse<T> {
    fn ok(data: T) -> Self {
        Self { status: true, data }
    }
}

impl JsonOkResponse<()> {
    fn status(status: bool) -> Self {
        Self { status, data: () }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct JsonErrResponse<T> {
    status: bool,
    error: T,
}

impl<T> JsonErrResponse<T> {
    fn err(error: T) -> Self {
        Self {
            status: false,
            error,
        }
    }
}
