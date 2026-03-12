pub(crate) mod error;
pub(crate) mod graph;
pub(crate) mod param;
mod schema;
use std::{path::PathBuf, sync::Arc};

use crate::schema::Query;
use actix_web::{
    App, HttpResponse, HttpServer, get, guard,
    http::StatusCode,
    post,
    web::{self, Json},
};
use async_graphql::{
    EmptyMutation, EmptySubscription, Name, ObjectType, OutputType, Schema, Value,
    dynamic::{Field, Subscription, TypeRef, indexmap::IndexMap},
    http::GraphiQLSource,
    parser::parse_schema,
};
use async_graphql_actix_web::{GraphQL, GraphQLRequest, GraphQLResponse};
use dotenv::from_filename;
use env_logger::{Builder, Env};
use error::GQLError;
use futures::executor::block_on;
use log::LevelFilter;
use reqwest::Response;
use serde::{Deserializer, de::Visitor};

// use crate::schema::get_schema;

#[post("/graphql")]
async fn query(
    req: GraphQLRequest,
    schema: web::Data<Schema<Query, EmptyMutation, EmptySubscription>>,
) -> Json<Value> {
    web::Json(schema.execute(req.into_inner()).await.data)
}

async fn graphiql_service() -> HttpResponse {
    HttpResponse::build(StatusCode::OK).body(
        GraphiQLSource::build()
            .version("2")
            .endpoint("/")
            .title("AusMetrics GraphQL Schema")
            .finish(),
    )
}

#[actix_web::main]
async fn main() -> Result<(), GQLError> {
    Builder::new()
        .format_source_path(true)
        .format_module_path(false)
        .format_target(false)
        .format_timestamp(None)
        .filter_module("graphql_server::schema", LevelFilter::Debug)
        // .filter_level(LevelFilter::Debug)
        .init();
    from_filename(".env.local").ok();

    let addr = || std::env::var("GRAPHQL_ADDR").unwrap();
    let port = u16::from_str_radix(std::env::var("PORT")?.as_str(), 10)?;

    let client = Arc::new(reqwest::Client::new());

    // let schema = get_schema().await;

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    HttpServer::new(move || {
        let schema_clone = schema.clone();
        let cors = actix_cors::Cors::default().allow_any_origin();
        App::new()
            .app_data(web::Data::new(schema_clone.clone()))
            .app_data(web::Data::new(client.clone()))
            .wrap(cors)
            .service(
                web::resource("/")
                    .guard(guard::Post())
                    .to(GraphQL::new(schema_clone)),
            )
            .service(web::resource("/").guard(guard::Get()).to(graphiql_service))
            .service(query)
    })
    .on_connect(move |_conn, _ext| {
        println!("now listening at http://{} on port {}", addr(), port);
    })
    .bind((addr(), port))?
    .run()
    .await?;
    Ok(())
}
