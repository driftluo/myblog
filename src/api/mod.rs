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

use salvo::{
    prelude::{async_trait, fn_handler},
    routing::FlowCtrl,
    Depot, Request, Response,
};
use tokio::sync::RwLock;

// todo: remove on delete all template
const PREFIX: &str = "api/v1/";

static PAGE_MAX: RwLock<PageMeta> = RwLock::const_new(PageMeta::new());

#[fn_handler]
async fn block_unlogin(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) -> Result<(), salvo::http::StatusError> {
    let permission = {
        depot
            .get::<Option<i16>>(crate::PERMISSION)
            .map(|a| a.is_some())
            .unwrap_or_default()
    };
    if permission {
        ctrl.call_next(req, depot, res).await;
        Ok(())
    } else {
        Err(crate::utils::from_code(
            salvo::hyper::StatusCode::FORBIDDEN,
            "No permission",
        ))
    }
}

#[fn_handler]
pub(crate) async fn block_no_admin(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) -> Result<(), salvo::http::StatusError> {
    let permission = {
        depot
            .get::<Option<i16>>(crate::PERMISSION)
            .map(|a| a.map(|b| b == 0))
            .flatten()
            .unwrap_or_default()
    };
    if permission {
        ctrl.call_next(req, depot, res).await;
        Ok(())
    } else {
        Err(crate::utils::from_code(
            salvo::hyper::StatusCode::FORBIDDEN,
            "No permission",
        ))
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

#[derive(Default, Copy, Clone)]
struct PageMeta {
    max_size: usize,
    // Base ten
    max_page: usize,
}

impl PageMeta {
    pub const fn new() -> Self {
        PageMeta {
            max_page: 0,
            max_size: 0,
        }
    }

    fn add(&mut self) {
        self.max_size += 1;
        self.max_page = self.max_size % 10;
    }

    fn reduce(&mut self) {
        self.max_size -= 1;
        self.max_page = self.max_size % 10;
    }
}

pub async fn init_page_size() {
    use crate::models::articles::ArticleList;

    let count = ArticleList::size_count().await;

    let mut page = PAGE_MAX.write().await;

    *page = PageMeta {
        max_size: count,
        max_page: count % 10,
    }
}

pub(crate) async fn size_add() {
    let mut page = PAGE_MAX.write().await;
    page.add()
}

pub(crate) async fn size_reduce() {
    let mut page = PAGE_MAX.write().await;
    page.reduce()
}

pub(crate) async fn current_size() -> usize {
    PAGE_MAX.read().await.max_size
}

pub(crate) async fn current_page() -> usize {
    PAGE_MAX.read().await.max_page
}
