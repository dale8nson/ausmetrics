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
// static FIELDS: HashSet<&str> = HashSet::<&str>::new();

// const STRUCTURE_TYPES: [&'static str; 8] = [
//     "datastructure",
//     "dataflow",
//     "codelist",
//     "conceptscheme",
//     "categoryscheme",
//     "contentconstraint",
//     "actualconstraint",
//     // "agencyscheme",
//     "categorisation",
//     // "hierarchicalcodelist",
// ];

// fn to_typeref(name: &str, value: Value) -> TypeRef {
//     // if name == "dataStructures" {
//     //     debug!("{name:?}: {value:?}");
//     // }
//     match value {
//         Value::Object(_) => TypeRef::named(name),
//         Value::List(list) => {
//             if let Some(first) = list.get(0) {
//                 // if the list contains objects, build a list of that object type;
//                 // otherwise build a list of the scalar type (e.g. Int, Float, String)
//                 match first {
//                     Value::Object(_) => TypeRef::named_list(name),
//                     _ => {
//                         let inner = to_typeref(name, first.clone());
//                         info!("{:?}", inner);
//                         TypeRef::List(Box::new(inner))
//                     }
//                 }
//             } else {
//                 // empty list ⇒ default to list of strings
//                 TypeRef::named_list(TypeRef::STRING)
//             }
//         }
//         Value::String(s) if s.to_lowercase() == "id" => TypeRef::named(TypeRef::ID),
//         Value::String(_) => TypeRef::named(TypeRef::STRING),
//         Value::Number(num) if num.is_f64() => TypeRef::named(TypeRef::FLOAT),
//         Value::Number(_) => TypeRef::named(TypeRef::INT),
//         Value::Boolean(_) => TypeRef::named(TypeRef::BOOLEAN),
//         Value::Binary(_) => TypeRef::named(TypeRef::UPLOAD),
//         Value::Enum(_) => TypeRef::named(TypeRef::STRING),
//         _ => TypeRef::named(TypeRef::STRING),
//     }
// }

// fn to_field(name: Name, value: Value, ty_override: Option<TypeRef>) -> Field {
//     let field_name = name.to_string();
//     // Use the override type if provided; otherwise, infer it from the JSON value.
//     let ty = ty_override.unwrap_or_else(|| to_typeref(&field_name, value.clone()));

//     // Remember if this field is a list type so we can return an empty list
//     // instead of `Null` when the JSON key is missing.
//     let is_list = matches!(ty, TypeRef::List(_));

//     Field::new(&field_name.clone(), ty, move |ctx| {
//         let field_name = field_name.clone();
//         let is_list = is_list;
//         FieldFuture::new(async move {
//             let parent_value = ctx.parent_value.as_value();

//             let child_value = if let Some(Value::Object(map)) = parent_value {
//                 map.get(&Name::new(&field_name))
//                     .cloned()
//                     .unwrap_or_else(|| {
//                         if is_list {
//                             Value::List(vec![])
//                         } else {
//                             Value::Null
//                         }
//                     })
//             } else if is_list {
//                 Value::List(vec![])
//             } else {
//                 Value::Null
//             };

//             Ok(Some(child_value))
//         })
//     })
// }

// fn to_object(type_name: Name, map: IndexMap<Name, Value>) -> (Object, Vec<Object>) {
//     debug!("{:?}", type_name.as_str());
//     let mut objs = Vec::<Object>::new();

//     if map.is_empty() {
//         return (Object::new(type_name.as_str()), objs);
//     }

//     let type_name_str = type_name.as_str().to_string();
//     let mut obj = Object::new(&type_name_str);
//     let mut fields = HashSet::<&str>::new();

//     for (field_name, value) in map.iter() {
//         let field_key = field_name.as_str();

//         debug!(
//             "{:?}: {:?}",
//             field_key,
//             to_typeref(field_key, value.clone())
//         );

//         match value.clone() {
//             Value::Object(index_map) => {
//                 if !index_map.is_empty() {
//                     // Construct a unique type name for this nested object based on the parent type.
//                     let nested_type_name = format!("{}_{}", type_name_str, field_key);
//                     let (nested_obj, nested_objs) =
//                         to_object(Name::new(&nested_type_name), index_map);

//                     objs.extend(nested_objs);

//                     if fields.insert(field_key) {
//                         let field_type = TypeRef::named(&nested_type_name);
//                         obj = obj.field(to_field(
//                             field_name.clone(),
//                             value.clone(),
//                             Some(field_type),
//                         ));
//                     }

//                     objs.push(nested_obj);
//                 }
//             }

//             Value::List(lst) => {
//                 // Handle empty lists first: we can't infer element structure.
//                 if lst.is_empty() {
//                     if fields.insert(field_key) {
//                         let list_type = TypeRef::named_list(TypeRef::STRING);
//                         obj = obj.field(to_field(
//                             field_name.clone(),
//                             Value::List(vec![]),
//                             Some(list_type),
//                         ));
//                     }
//                     continue;
//                 }

//                 match lst[0].clone() {
//                     Value::Object(index_map) => {
//                         if !index_map.is_empty() {
//                             // Unique type name for list element objects as well.
//                             let nested_type_name = format!("{}_{}", type_name_str, field_key);
//                             let (nested_obj, nested_objs) =
//                                 to_object(Name::new(&nested_type_name), index_map);

//                             objs.extend(nested_objs);

//                             if fields.insert(field_key) {
//                                 let field_type = TypeRef::named_list(&nested_type_name);
//                                 obj = obj.field(to_field(
//                                     field_name.clone(),
//                                     value.clone(),
//                                     Some(field_type),
//                                 ));
//                             }

//                             objs.push(nested_obj);
//                         }
//                     }
//                     _ => {
//                         // List of scalars: let to_typeref infer the scalar type.
//                         if fields.insert(field_key) {
//                             obj = obj.field(to_field(field_name.clone(), value.clone(), None));
//                         }
//                     }
//                 }
//             }

//             _ => {
//                 debug!(
//                     "field: {:?}: {:?}",
//                     field_name.as_str(),
//                     to_typeref(field_name.as_str(), value.clone())
//                 );
//                 if fields.insert(field_key) {
//                     obj = obj.field(to_field(field_name.clone(), value.clone(), None));
//                 }
//             }
//         }
//     }

//     debug!("obj: {:?}", obj.type_name());
//     (obj, objs)
// }

// pub async fn get_schema() -> Schema {
//     let path = PathBuf::from("https://data.api.abs.gov.au/rest");
//     let client = Arc::new(reqwest::Client::new());

//     let mut query = Object::new("Query");
//     let mut schema = Schema::build(query.type_name(), None, None);
//     let mut meta_map = IndexMap::<Name, Value>::new();
//     // Custom root meta type that aggregates ABS structure types.
//     // Use a unique name so it does not collide with any "meta" objects
//     // that may appear inside the ABS JSON and be turned into GraphQL
//     // types by `to_object`.
//     let mut root_meta = Object::new("AbsStructuresMeta");

//     for ty in STRUCTURE_TYPES.iter() {
//         debug!("ty: {ty}");
//         let type_name = *ty;
//         let mut url = path.clone();

//         url.push(format!("{ty}/ABS?detail=allstubs"));
//         debug!("url: {url:?}");

//         let res = client
//             .get(url.to_str().unwrap())
//             .header("accept", "application/vnd.sdmx.structure+json")
//             .header("user-agent", "ausmetrics/0.1.0")
//             .send()
//             .await
//             .expect("Error retrieving data");

//         let value = res.json::<Value>().await.expect("Unable to parse JSON");

//         meta_map.insert(Name::new(type_name), value.clone());

//         if let Value::Object(ref index_map) = value.clone() {
//             let (ob, obs) = to_object(Name::new(type_name), index_map.clone());
//             schema = schema.register(ob);
//             for o in obs {
//                 schema = schema.register(o);
//             }

//             // add this structure as a field on the root meta object
//             let v = value.clone();
//             root_meta = root_meta.field(to_field(Name::new(type_name), v, None));
//         }
//     }

//     let meta_data = meta_map.clone();
//     // Expose the aggregated structures under Query.meta, whose type is AbsStructuresMeta
//     query = query.field(Field::new(
//         "meta",
//         TypeRef::named("AbsStructuresMeta"),
//         move |_| FieldFuture::from_value(Some(Value::Object(meta_data.clone()))),
//     ));

//     schema = schema
//         .data(client.clone())
//         .register(query)
//         .register(root_meta);
//     let schema = schema.finish().expect("unable to parse schema");
//     let dataflow_ids = schema
//         .execute("{meta {dataflow { data { dataflows { id }}}}}")
//         .await
//         .data;

//     schema
// }

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
                *state = Quarter {
                    year: state.year + ((state.quarter + 1) % 4 + 1) / 4,
                    quarter: (state.quarter + 1) % 4 + 1,
                };
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
    year: u16,
    quarter: u16,
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
        let values = js_path_vals("$.data.dataSets[0].series..observations", &json)
            .unwrap_or(vec![&Value::Null]);

        if let Value::Object(map) = &mut values[0].clone() {
            Value::Object(
                map.iter()
                    .scan(start.clone(), |state, (_, value)| {
                        let date = NaiveDate::default()
                            .with_year(state.year)
                            .unwrap_or_default()
                            .with_month(state.month)
                            .unwrap_or_default()
                            .format("%b-%Y")
                            .to_string();

                        *state = Period {
                            year: state.year + (state.month / 12) as i32,
                            month: (state.month) % 12 + 1,
                        };
                        Some((date, value[0].clone()))
                    })
                    .collect::<Map<String, Value>>(),
            )
        } else {
            Value::Null
        }
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
