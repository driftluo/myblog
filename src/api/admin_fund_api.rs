use http_body_util::BodyExt;
use salvo::{
    Request, Response, Router,
    http::{StatusCode, StatusError},
    prelude::handler,
};

use crate::{
    Routers,
    api::{JsonErrResponse, JsonOkResponse, block_no_admin},
    models::fund::{
        BatchUpdateEntry, FundEntry, FundPortfolio, NewFundEntry, NewPortfolio, PortfolioSimple,
        PortfolioWithEntriesSimple, UpdateFundEntry, UpdatePortfolio,
    },
    utils::{from_code, parse_json_body, parse_last_path, set_json_response},
};

// ============== Portfolio Handlers ==============

#[handler]
async fn list_portfolios(res: &mut Response) -> Result<(), StatusError> {
    match FundPortfolio::list_all().await {
        Ok(data) => {
            let simple: Vec<PortfolioSimple> = data.into_iter().map(|p| p.into()).collect();
            set_json_response(res, 256, JsonOkResponse::ok(simple))
        }
        Err(e) => set_json_response(res, 64, JsonErrResponse::err(e)),
    }
    Ok(())
}

#[handler]
async fn get_portfolio(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let id = parse_last_path::<i32>(req)?;

    match PortfolioWithEntriesSimple::get(id).await {
        Ok(data) => set_json_response(res, 1024, JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 64, JsonErrResponse::err(e)),
    }
    Ok(())
}

#[handler]
async fn create_portfolio(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let body = parse_json_body::<NewPortfolio>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    match FundPortfolio::create(body).await {
        Ok(id) => set_json_response(res, 64, JsonOkResponse::ok(id)),
        Err(e) => set_json_response(res, 64, JsonErrResponse::err(e)),
    }
    Ok(())
}

#[handler]
async fn update_portfolio(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let body = parse_json_body::<UpdatePortfolio>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    match FundPortfolio::update(body).await {
        Ok(_) => set_json_response(res, 32, JsonOkResponse::status(true)),
        Err(e) => set_json_response(res, 64, JsonErrResponse::err(e)),
    }
    Ok(())
}

#[handler]
async fn delete_portfolio(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let id = parse_last_path::<i32>(req)?;

    match FundPortfolio::delete(id).await {
        Ok(_) => set_json_response(res, 32, JsonOkResponse::status(true)),
        Err(e) => set_json_response(res, 64, JsonErrResponse::err(e)),
    }
    Ok(())
}

// ============== Entry Handlers ==============

#[handler]
async fn list_entries(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let portfolio_id = parse_last_path::<i32>(req)?;

    match FundEntry::list_by_portfolio(portfolio_id).await {
        Ok(data) => set_json_response(res, 1024, JsonOkResponse::ok(data)),
        Err(e) => set_json_response(res, 64, JsonErrResponse::err(e)),
    }
    Ok(())
}

#[handler]
async fn create_entry(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    // Read body once, then try to deserialize as array or single object
    let body_bytes = {
        let body = req.take_body();
        let collected = match body.collect().await {
            Ok(c) => c,
            Err(_) => return Err(from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect")),
        };
        collected.to_bytes()
    };

    // Try array first
    if let Ok(list) = serde_json::from_slice::<Vec<NewFundEntry>>(&body_bytes) {
        let mut ids: Vec<i32> = Vec::with_capacity(list.len());
        for item in list {
            match FundEntry::create(item).await {
                Ok(id) => ids.push(id),
                Err(e) => {
                    return {
                        let _: () = set_json_response(res, 64, JsonErrResponse::err(e));
                        Ok(())
                    };
                }
            }
        }
        set_json_response(res, 256, JsonOkResponse::ok(ids));
        return Ok(());
    }

    // Try single object
    if let Ok(item) = serde_json::from_slice::<NewFundEntry>(&body_bytes) {
        match FundEntry::create(item).await {
            Ok(id) => set_json_response(res, 64, JsonOkResponse::ok(id)),
            Err(e) => set_json_response(res, 64, JsonErrResponse::err(e)),
        }
        return Ok(());
    }

    // Bad JSON
    return Err(from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"));
}

#[handler]
async fn update_entry(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    // Read body once and attempt to deserialize as array or single object
    let body_bytes = {
        let body = req.take_body();
        let collected = match body.collect().await {
            Ok(c) => c,
            Err(_) => return Err(from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect")),
        };
        collected.to_bytes()
    };

    if let Ok(list) = serde_json::from_slice::<Vec<UpdateFundEntry>>(&body_bytes) {
        let mut portfolio_id: Option<i32> = None;
        for item in list {
            match FundEntry::update(item).await {
                Ok(pid) => portfolio_id = Some(pid),
                Err(e) => {
                    set_json_response(res, 64, JsonErrResponse::err(e));
                    return Ok(());
                }
            }
        }
        return {
            set_json_response(res, 64, JsonOkResponse::ok(portfolio_id));
            Ok(())
        };
    }

    if let Ok(item) = serde_json::from_slice::<UpdateFundEntry>(&body_bytes) {
        match FundEntry::update(item).await {
            Ok(portfolio_id) => set_json_response(res, 64, JsonOkResponse::ok(portfolio_id)),
            Err(e) => set_json_response(res, 64, JsonErrResponse::err(e)),
        }
        return Ok(());
    }

    return Err(from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"));
}

#[handler]
async fn delete_entry(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    // Single delete via path parameter
    let id = parse_last_path::<i32>(req)?;

    match FundEntry::delete(id).await {
        Ok(portfolio_id) => set_json_response(res, 64, JsonOkResponse::ok(portfolio_id)),
        Err(e) => set_json_response(res, 64, JsonErrResponse::err(e)),
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct BatchUpdateRequest {
    portfolio_id: i32,
    updates: Vec<BatchUpdateEntry>,
}

#[derive(serde::Deserialize)]
struct BatchUpdateOrderRequest {
    portfolio_id: i32,
    updates: Vec<crate::models::fund::BatchUpdateOrderEntry>,
}

#[handler]
async fn batch_update_amounts(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let body = parse_json_body::<BatchUpdateRequest>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    match FundEntry::batch_update_amounts(body.portfolio_id, body.updates).await {
        Ok(_) => set_json_response(res, 32, JsonOkResponse::status(true)),
        Err(e) => set_json_response(res, 64, JsonErrResponse::err(e)),
    }
    Ok(())
}

#[handler]
async fn batch_update_order(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let body = parse_json_body::<BatchUpdateOrderRequest>(req)
        .await
        .ok_or_else(|| from_code(StatusCode::BAD_REQUEST, "Json body is Incorrect"))?;

    match FundEntry::batch_update_order(body.portfolio_id, body.updates).await {
        Ok(_) => set_json_response(res, 32, JsonOkResponse::status(true)),
        Err(e) => set_json_response(res, 64, JsonErrResponse::err(e)),
    }
    Ok(())
}

// ============== Router ==============

pub struct AdminFund;

impl Routers for AdminFund {
    fn build(self) -> Vec<Router> {
        use crate::api::PREFIX;
        vec![
            Router::new()
                .path(PREFIX.to_owned() + "fund")
                .hoop(block_no_admin)
                // Portfolio routes
                // GET /api/v1/fund/portfolios - list all portfolios
                .push(Router::new().path("portfolios").get(list_portfolios))
                // GET /api/v1/fund/portfolio/{id} - get portfolio with entries
                .push(Router::new().path("portfolio/{id}").get(get_portfolio))
                // POST /api/v1/fund/portfolio - create new portfolio
                .push(Router::new().path("portfolio").post(create_portfolio))
                // POST /api/v1/fund/portfolio/update - update portfolio
                .push(
                    Router::new()
                        .path("portfolio/update")
                        .post(update_portfolio),
                )
                // POST /api/v1/fund/portfolio/delete/{id} - delete portfolio
                .push(
                    Router::new()
                        .path("portfolio/delete/{id}")
                        .post(delete_portfolio),
                )
                // Entry routes
                // GET /api/v1/fund/entries/{portfolio_id} - list entries for portfolio
                .push(
                    Router::new()
                        .path("entries/{portfolio_id}")
                        .get(list_entries),
                )
                // POST /api/v1/fund/entry - create new entry
                .push(Router::new().path("entry").post(create_entry))
                // POST /api/v1/fund/entry/update - update entry
                .push(Router::new().path("entry/update").post(update_entry))
                // POST /api/v1/fund/entry/delete/{id} - delete entry
                .push(Router::new().path("entry/delete/{id}").post(delete_entry))
                // POST /api/v1/fund/entries/batch-update - batch update amounts
                .push(
                    Router::new()
                        .path("entries/batch-update")
                        .post(batch_update_amounts),
                )
                // POST /api/v1/fund/entries/batch-order - batch update order
                .push(
                    Router::new()
                        .path("entries/batch-order")
                        .post(batch_update_order),
                ),
        ]
    }
}
