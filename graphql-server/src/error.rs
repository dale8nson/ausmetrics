use async_graphql::dynamic::SchemaError;
use std::error::Error as StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GQLError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    DotEnv(#[from] dotenv::Error),
    #[error(transparent)]
    EnvVar(#[from] std::env::VarError),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ScanError(#[from] yaml_rust2::scanner::ScanError),
    #[error(transparent)]
    Deserialize(#[from] serde::de::value::Error),
    #[error(transparent)]
    Serializer(#[from] async_graphql_value::SerializerError),
    #[error(transparent)]
    JsonSerialize(#[from] serde_json::error::Error),
    #[error("Error source: {}", .0.cause().unwrap())]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
