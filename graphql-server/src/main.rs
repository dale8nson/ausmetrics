pub(crate) mod error;
mod schema;
use std::{path::PathBuf, str::FromStr};

use actix_web::{App, HttpResponse, HttpServer, get, guard, http::StatusCode, post, web};
use async_graphql::{dynamic::Schema, http::GraphiQLSource};
use async_graphql_actix_web::{GraphQL, GraphQLRequest, GraphQLResponse};
use dotenv::from_filename;
use error::GQLError;

use crate::schema::{parse_yaml_doc, to_gql};

// #[post("/")]
// async fn gql_req(req: GraphQLRequest, schema: web::Data<Schema>) -> web::Json<GraphQLResponse> {
//     web::Json(schema.execute(req.into_inner()).await.into())
// }

#[get("/graphiql")]
async fn graphiql_service() -> HttpResponse {
    HttpResponse::build(StatusCode::OK).body(
        GraphiQLSource::build()
            .endpoint("/grahpiql")
            .title("AusMetrics graphql schema")
            .finish(),
    )
}

#[actix_web::main]
async fn main() -> Result<(), GQLError> {
    from_filename(".env.local").ok();

    let addr = || std::env::var("GRAPHQL_ADDR").unwrap();
    let port = u16::from_str_radix(std::env::var("PORT")?.as_str(), 10)?;

    let yaml = parse_yaml_doc(PathBuf::from_str("specifications/sdmx-rest.yaml").unwrap())?;
    let schema = to_gql(yaml)?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .service(
                web::resource("/")
                    .guard(guard::Post())
                    .to(GraphQL::new(schema.clone())),
            )
            .service(graphiql_service)
    })
    .on_connect(move |_conn, _ext| {
        println!("now listening at http://{} on port {}", addr(), port);
    })
    .bind((addr(), port))?
    .run()
    .await?;
    Ok(())
}
