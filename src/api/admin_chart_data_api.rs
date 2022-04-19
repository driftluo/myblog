use salvo::{
    http::StatusError,
    prelude::{async_trait, fn_handler},
    Request, Response, Router,
};

use crate::{
    api::{block_no_admin, JsonErrResponse, JsonOkResponse},
    db_wrapper::get_redis,
    models::articles::PublishedStatistics,
    utils::{parse_query, set_json_response},
    Routers,
};

#[fn_handler]
async fn publish_by_month(res: &mut Response) {
    match PublishedStatistics::statistics_published_frequency_by_month().await {
        Ok(data) => set_json_response(res, 128, &JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 32, &JsonErrResponse::err(e)),
    }
}

#[fn_handler]
async fn get_ip_chart(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let limit = parse_query::<i64>(req, "limit")?;
    let offset = parse_query::<i64>(req, "offset")?;

    let data = get_redis()
        .lrange::<Vec<String>>("visitor_log", offset, offset + limit - 1)
        .await;

    set_json_response(res, 128, &JsonOkResponse::ok(data));
    Ok(())
}

pub struct ChartData;

impl Routers for ChartData {
    fn build(self) -> Vec<Router> {
        use crate::api::PREFIX;
        vec![
            // http {ip}/article/month
            Router::new()
                .path(PREFIX.to_owned() + "article/month")
                .hoop(block_no_admin)
                .get(publish_by_month),
            // http {ip}/ip/view limit==5 offset==0
            Router::new()
                .path(PREFIX.to_owned() + "ip/view")
                .hoop(block_no_admin)
                .get(get_ip_chart),
        ]
    }
}
