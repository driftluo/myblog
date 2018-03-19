use sapper::{Error as SapperError, Request, Response, Result as SapperResult, SapperModule,
             SapperRouter};
use sapper_std::{QueryParams, SessionVal};

use super::super::{admin_verification_cookie, Postgresql, PublishedStatistics, Redis};

pub struct ChartData;

impl ChartData {
    fn publish_by_month(req: &mut Request) -> SapperResult<Response> {
        let pg_pool = req.ext().get::<Postgresql>().unwrap().get().unwrap();
        let res = match PublishedStatistics::statistics_published_frequency_by_month(&pg_pool) {
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

    fn get_ip_chart(req: &mut Request) -> SapperResult<Response> {
        let params = get_query_params!(req);
        let limit = t_param_parse!(params, "limit", i64);
        let offset = t_param_parse!(params, "offset", i64);
        let redis_pool = req.ext().get::<Redis>().unwrap();
        let res = json!({
                "status": true,
                "data": redis_pool.lrange::<Vec<String>>("visitor_log", 0 + offset, 0 + offset + limit - 1)
        });
        res_json!(res)
    }
}

impl SapperModule for ChartData {
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
        router.get("/article/month", ChartData::publish_by_month);

        router.get("/ip/view", ChartData::get_ip_chart);

        Ok(())
    }
}
