use juiz_core::Value;
use utoipa::OpenApi;

use super::IdentifierQuery;
use axum::{extract::Query, Json};


#[utoipa::path(
    get,
    path = "/api/container_process/profile_full",
    params(
        IdentifierQuery
    ),
    responses(
        (status = 200, description = "System")
    ),
    tag = "container_process",
)]
pub fn profile_handler_dummy(
    _query: Query<IdentifierQuery>,) {
}

#[utoipa::path(
    get,
    path = "/api/container_process/list",
    responses(
        (status = 200, description = "System")
    ),
    tag = "container_process",
)]
pub fn list_dummy() {
}

#[utoipa::path(
    patch,
    path = "/api/container_process/call",
    params(
        IdentifierQuery
    ),
    request_body = Value,
    responses(
        (status = 200, description = "System")
    ),
    tag = "container_process",
)]
pub fn call_dummy(
_query: Query<IdentifierQuery>,
Json(_body): Json<Value>) {
}


#[derive(OpenApi)]
#[openapi(
    paths(
        profile_handler_dummy,
        list_dummy,
        call_dummy,
    ),
    components(schemas(
    ))
)]
pub struct ApiDoc;