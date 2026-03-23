use std::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt::{Display, Write},
    io::BufReader,
    ops::Deref,
    path::PathBuf,
    rc::Rc,
    str::FromStr,
    sync::{Arc, Mutex, RwLock},
    vec::IntoIter,
};

use actix_web::Either;
use bytes::Bytes;
use chrono::{Datelike, Local, NaiveDate};
use csv::{ByteRecord, Reader, ReaderBuilder};
use futures::{StreamExt, stream};
use log::{debug, info};

use async_graphql::{
    InputType,
    dynamic::{
        Field, FieldFuture, FieldValue, Object, Scalar, Schema, SchemaBuilder, TypeRef,
        indexmap::IndexMap,
    },
    *,
};
use jsonpath_rust::query::js_path_vals;
use reqwest::Client;
use serde_json::{Map, Value};

use crate::ffi;

async fn fetch_from_abs(url: PathBuf, client: &Client) -> Value {
    let res = client
        .get(url.to_str().unwrap())
        .header("accept", "application/vnd.sdmx.data+json")
        .header("user-agent", "ausmetrics/0.1.0")
        .send()
        .await;

    res.unwrap().json().await.unwrap_or(Value::Null)
}

fn to_quarterly_data(value: &Value, start: &Quarter) -> Value {
    debug!("value: {value:#?}");
    let map = match value.clone() {
        Value::Object(map) => map,
        _ => Map::<String, Value>::new(),
    };

    Value::Object(
        map.iter()
            .scan(start.clone(), |state, (_k, v)| {
                let key = format!("{}-Q{}", state.year, state.quarter);
                let value = if let Value::Array(arr) = v {
                    arr[0].clone()
                } else {
                    Value::Number(Number::from_f64(0.0).unwrap())
                };
                state.year = state.year + (state.quarter / 4) as i32;
                state.quarter = (state.quarter + 1) % 4;

                // *state = Quarter {
                //     year: state.year + (state.quarter / 4) as i32,
                //     quarter: (state.quarter + 1) % 4,
                // };
                Some((key, value))
            })
            .collect::<Map<String, Value>>(),
    )
}

async fn fetch_cash_rate_targets() -> Value {
    let date_time = Local::now();
    let date_time_str = date_time.format("%Y-%m-%d-%H-%M-%S").to_string();

    let url = PathBuf::from(format!(
        "https://www.rba.gov.au/statistics/tables/csv/f1.1-data.csv?v={}",
        date_time_str
    ));

    let res = reqwest::get(url.to_str().unwrap()).await;

    if let Ok(res) = res {
        let text = res.text().await.unwrap_or_default();
        let bytes = text.as_bytes();
        let reader = ReaderBuilder::new().flexible(true).from_reader(&*bytes);

        Value::Object(
            reader
                .into_records()
                .skip_while(|r| &r.as_ref().unwrap()[0] != "31/08/1990")
                .map(|r| {
                    let r = r.unwrap_or_default();

                    let date = NaiveDate::parse_from_str(&r[0], "%d/%m/%Y")
                        .unwrap_or_default()
                        .format("%b-%Y-%d")
                        .to_string();

                    let value = Value::Number(
                        Number::from_str(&r[1]).unwrap_or(Number::from_f64(0.0).unwrap()),
                    );

                    (date, value)
                })
                .collect::<Map<String, Value>>(),
        )
    } else {
        Value::Null
    }
}

#[derive(InputObject, SimpleObject, Debug, Clone)]
struct Period {
    year: i32,
    month: u32,
}

#[derive(InputObject, Ord, PartialOrd, PartialEq, Eq, Debug, Clone)]
struct Quarter {
    year: i32,
    quarter: u32,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone)]
pub struct MeanHousePrice {
    start: Quarter,
    end: Quarter,
}

#[Object]
impl MeanHousePrice {
    #[graphql(name = "NSW")]
    async fn nsw<'ctx>(&self, ctx: &Context<'ctx>) -> Value {
        let client = ctx.data::<Client>().cloned().unwrap_or(Client::default());

        let url = PathBuf::from(format!(
            "https://data.api.abs.gov.au/rest/data/ABS,RES_DWELL_ST,1.0.0/5.1.Q?startPeriod={}-Q{}&endPeriod={}-Q{}",
            self.start.year, self.start.quarter, self.end.year, self.end.quarter
        ));

        let json = fetch_from_abs(url, &client).await;

        let parsed_json = unsafe {
            ffi::abs_parse_sdmx(
                std::ffi::CString::new(json.to_string())
                    .expect("CString::new failed")
                    .as_ptr(),
            )
        };

        debug!("{parsed_json:#?}");

        let value =
            js_path_vals("$.data.dataSets[0].series..observations", &json).unwrap_or(vec![])[0];

        to_quarterly_data(value, &self.start)
    }

    #[graphql(name = "VIC")]
    async fn vic<'ctx>(&self, ctx: &Context<'ctx>) -> Value {
        let client = ctx.data::<Client>().cloned().unwrap_or(Client::default());

        let url = PathBuf::from(format!(
            "https://data.api.abs.gov.au/rest/data/ABS,RES_DWELL_ST,1.0.0/5.2.Q?startPeriod={}-Q{}&endPeriod={}-Q{}",
            self.start.year, self.start.quarter, self.end.year, self.end.quarter
        ));

        let json = fetch_from_abs(url, &client).await;

        let value =
            js_path_vals("$.data.dataSets[0].series..observations", &json).unwrap_or(vec![])[0];

        to_quarterly_data(value, &self.start)
    }

    #[graphql(name = "Qu")]
    async fn qu<'ctx>(&self, ctx: &Context<'ctx>) -> Value {
        let client = ctx.data::<Client>().cloned().unwrap_or(Client::default());

        let url = PathBuf::from(format!(
            "https://data.api.abs.gov.au/rest/data/ABS,RES_DWELL_ST,1.0.0/5.3.Q?startPeriod={}-Q{}&endPeriod={}-Q{}",
            self.start.year, self.start.quarter, self.end.year, self.end.quarter
        ));

        let json = fetch_from_abs(url, &client).await;

        let value =
            js_path_vals("$.data.dataSets[0].series..observations", &json).unwrap_or(vec![])[0];

        to_quarterly_data(value, &self.start)
    }

    #[graphql(name = "SA")]
    async fn sa<'ctx>(&self, ctx: &Context<'ctx>) -> Value {
        let client = ctx.data::<Client>().cloned().unwrap_or(Client::default());

        let url = PathBuf::from(format!(
            "https://data.api.abs.gov.au/rest/data/ABS,RES_DWELL_ST,1.0.0/5.4.Q?startPeriod={}-Q{}&endPeriod={}-Q{}",
            self.start.year, self.start.quarter, self.end.year, self.end.quarter
        ));

        let json = fetch_from_abs(url, &client).await;

        let value =
            js_path_vals("$.data.dataSets[0].series..observations", &json).unwrap_or(vec![])[0];

        to_quarterly_data(value, &self.start)
    }

    #[graphql(name = "WA")]
    async fn wa<'ctx>(&self, ctx: &Context<'ctx>) -> Value {
        let client = ctx.data::<Client>().cloned().unwrap_or(Client::default());

        let url = PathBuf::from(format!(
            "https://data.api.abs.gov.au/rest/data/ABS,RES_DWELL_ST,1.0.0/5.5.Q?startPeriod={}-Q{}&endPeriod={}-Q{}",
            self.start.year, self.start.quarter, self.end.year, self.end.quarter
        ));

        let json = fetch_from_abs(url, &client).await;

        let value =
            js_path_vals("$.data.dataSets[0].series..observations", &json).unwrap_or(vec![])[0];

        to_quarterly_data(value, &self.start)
    }

    #[graphql(name = "Tas")]
    async fn tas<'ctx>(&self, ctx: &Context<'ctx>) -> Value {
        let client = ctx.data::<Client>().cloned().unwrap_or(Client::default());

        let url = PathBuf::from(format!(
            "https://data.api.abs.gov.au/rest/data/ABS,RES_DWELL_ST,1.0.0/5.6.Q?startPeriod={}-Q{}&endPeriod={}-Q{}",
            self.start.year, self.start.quarter, self.end.year, self.end.quarter
        ));

        let json = fetch_from_abs(url, &client).await;

        let value =
            js_path_vals("$.data.dataSets[0].series..observations", &json).unwrap_or(vec![])[0];

        to_quarterly_data(value, &self.start)
    }

    #[graphql(name = "NT")]
    async fn nt<'ctx>(&self, ctx: &Context<'ctx>) -> Value {
        let client = ctx.data::<Client>().cloned().unwrap_or(Client::default());

        let url = PathBuf::from(format!(
            "https://data.api.abs.gov.au/rest/data/ABS,RES_DWELL_ST,1.0.0/5.7.Q?startPeriod={}-Q{}&endPeriod={}-Q{}",
            self.start.year, self.start.quarter, self.end.year, self.end.quarter
        ));

        let json = fetch_from_abs(url, &client).await;

        let value =
            js_path_vals("$.data.dataSets[0].series..observations", &json).unwrap_or(vec![])[0];

        to_quarterly_data(value, &self.start)
    }

    #[graphql(name = "AUS")]
    async fn aus<'ctx>(&self, ctx: &Context<'ctx>) -> Value {
        let client = ctx.data::<Client>().cloned().unwrap_or(Client::default());

        let url = PathBuf::from(format!(
            "https://data.api.abs.gov.au/rest/data/ABS,RES_DWELL_ST,1.0.0/5.AUS.Q?startPeriod={}-Q{}&endPeriod={}-Q{}",
            self.start.year, self.start.quarter, self.end.year, self.end.quarter
        ));

        let json = fetch_from_abs(url, &client).await;

        let value =
            js_path_vals("$.data.dataSets[0].series..observations", &json).unwrap_or(vec![])[0];

        to_quarterly_data(value, &self.start)
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone)]
pub struct GrossDomesticProduct {
    start: Quarter,
    end: Quarter,
}

#[Object]
impl GrossDomesticProduct {
    async fn current_prices<'ctx>(&self, ctx: &Context<'ctx>) -> Value {
        let client = ctx.data::<Client>().cloned().unwrap_or(Client::default());

        let url = PathBuf::from(format!(
            "https://data.api.abs.gov.au/rest/data/ABS,ANA_AGG,1.0.0/M3.GPM...Q?startPeriod={}-Q{}&endPeriod={}-Q{}",
            self.start.year, self.start.quarter, self.end.year, self.end.quarter
        ));

        let json = fetch_from_abs(url, &client).await;
        debug!("json: {json:#?}");

        let values = js_path_vals("$.data.dataSets[*].series..observations.*", &json)
            .unwrap_or(vec![&Value::Null]);

        let values = Value::Object(
            values
                .into_iter()
                .enumerate()
                .map(|(idx, value)| (idx.to_string(), value.clone()))
                .collect::<Map<String, Value>>(),
        );

        debug!("values: {values:#?}");

        to_quarterly_data(&values, &self.start)
        // value.clone()
        // Value::Null
    }
}

pub struct Query;

#[Object]
impl Query {
    /// Cash Rate Target (%)
    /// Source: RBA
    async fn cash_rate_target<'ctx>(
        &self,
        _ctx: &Context<'ctx>,
        start: Period,
        end: Period,
    ) -> Value {
        let cash_rates = fetch_cash_rate_targets().await;
        let start_date = NaiveDate::default()
            .with_year(start.year)
            .unwrap()
            .with_month(start.month)
            .unwrap_or_default();
        let end_date = NaiveDate::default()
            .with_year(end.year)
            .unwrap_or_default()
            .with_month(end.month)
            .unwrap_or_default();

        if let Value::Object(map) = cash_rates {
            let cash_rates = map
                .into_iter()
                .filter(|(date, _)| {
                    let date =
                        NaiveDate::parse_from_str(date.as_str(), "%b-%Y-%d").unwrap_or_default();

                    date.cmp(&start_date).is_ge() && date.cmp(&end_date).is_le()
                })
                .map(|(date, value)| {
                    (
                        NaiveDate::parse_from_str(date.as_str(), "%b-%Y-%d")
                            .unwrap()
                            .format("%b-%Y")
                            .to_string(),
                        value,
                    )
                })
                .collect::<Map<String, Value>>();
            Value::Object(cash_rates)
        } else {
            Value::Null
        }
    }
    /// Consumer Price Index
    /// Source: ABS
    async fn consumer_price_index<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        start: Period,
        end: Period,
    ) -> Value {
        let client = ctx.data::<Client>().cloned().unwrap_or(Client::default());

        let url = PathBuf::from_str(format!("https://data.api.abs.gov.au/rest/data/ABS,CPI,2.0.0/3...50.M?startPeriod={}-{:02}&endPeriod={}-{:02}", start.year, start.month, end.year, end.month).as_str()).unwrap();

        let res = client
            .get(url.to_str().unwrap())
            .header("accept", "application/vnd.sdmx.data+json")
            .header("user-agent", "ausmetrics/0.1.0")
            .send()
            .await;

        let json = res.unwrap().json().await.unwrap_or(Value::Null);

        let parsed_json: Value = unsafe {
            let c_string = std::ffi::CString::new(json.to_string()).expect("CString::new failed");
            let mut parsed = ffi::abs_parse_sdmx(c_string.as_ptr());
            let mut s = String::new();
            while *parsed as u8 as char != '\0' {
                s.push(char::from_u32(*parsed as u32).unwrap());
                parsed = parsed.add(1);
            }

            s.parse().unwrap()
        };

        debug!("\x1b[1;36m{parsed_json:#?}\x1b[0m");
        // debug!("\x1b[1;31mparsed_json: {parsed_json:#?}\x1b[0m");

        // let values = js_path_vals("$.data", &parsed_json[0]).unwrap_or(vec![&Value::Null]);

        // debug!("values: {values:#?}");

        // if let Value::Object(map) = &mut values[0].clone() {
        //     Value::Object(
        //         map.iter()
        //             .scan(start.clone(), |state, (_, value)| {
        //                 let date = NaiveDate::default()
        //                     .with_year(state.year)
        //                     .unwrap_or_default()
        //                     .with_month(state.month)
        //                     .unwrap_or_default()
        //                     .format("%b-%Y")
        //                     .to_string();

        //                 *state = Period {
        //                     year: state.year + (state.month / 12) as i32,
        //                     month: (state.month + 1) % 12,
        //                 };
        //                 Some((date, value[0].clone()))
        //             })
        //             .collect::<Map<String, Value>>(),
        //     )

        let Value::Array(arr) = parsed_json else {
            return Value::Null;
        };
        let map = arr
            .into_iter()
            .map(|obj| {
                (
                    obj.get("period").unwrap().as_str().unwrap().to_string(),
                    obj.get("value").unwrap().clone(),
                )
            })
            .collect::<Map<String, Value>>();

        Value::Object(map)
    }

    /// Residential Dwellings: Mean Price by State and Territories
    /// Source: ABS
    async fn mean_house_price<'ctx>(
        &self,
        _ctx: &Context<'ctx>,
        start: Quarter,
        end: Quarter,
    ) -> MeanHousePrice {
        MeanHousePrice { start, end }
    }

    /// Gross Domestic Product (Current Prices)
    /// Source: ABS
    async fn gross_domestic_product<'ctx>(
        &self,
        _ctx: &Context<'ctx>,
        start: Quarter,
        end: Quarter,
    ) -> GrossDomesticProduct {
        GrossDomesticProduct { start, end }
    }
}
