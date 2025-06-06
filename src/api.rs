// api.rs
use crate::quote::{self, JsonQuote};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http,
    response::IntoResponse,
    routing::get,
    Router,

};
use std::sync::Arc;
use tokio::sync::RwLock;



pub fn router() -> Router<Arc<RwLock<AppState>>> 
{
    Router::new()
        .route("/quote/{quote_id}", get(get_quote_api))
        .route("/random-quote", get(get_random_quote_api))
}



async fn get_quote_data_for_api(
    db: &sqlx::SqlitePool,
    quote_id: &str,
) -> Result <impl IntoResponse, http::StatusCode> {
    match quote::get_quote_by_id_from_db(db, quote_id).await {
        Ok(quote_obj) => {
            let json_response = JsonQuote::new(&quote_obj);
            Ok(axum::Json(json_response).into_response())
        }
        Err(e) => {
            tracing::warn!("API: quote fetch failed for id {}: {}", quote_id, e);
            if matches!(e,sqlx::Error::RowNotFound) {
                Err(http::StatusCode::NOT_FOUND)
            } else {
                Err(http::StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/quote/{quote_id}",
    responses(
        (status = 200, description = "Get a quote by id", body = JsonQuote),
        (status = 404, description = "No matching quote found"),
        (status = 500, description = "Internal server error")

    ),
    params(
        ("quote_id" = String, Path, description = "ID of the quote to retrieve")
    )

)]


pub async fn get_quote_api(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(quote_id): Path<String>,
) -> impl IntoResponse {
    let state_guard = app_state.read().await;
    get_quote_data_for_api(&state_guard.db, &quote_id).await
}

#[utoipa::path(
    get,
    path = "/api/v1/random-quote",
    responses(
        (status = 200, description = "Get a random quote", body = JsonQuote),
        (status = 404, description = "No quotes available in the database"),

        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_random_quote_api(
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    let state_guard = app_state.read().await;
    let db_pool = &state_guard.db;
    match quote::get_random_quote_id_from_db(db_pool).await {
        Ok(found_quote_id) => get_quote_data_for_api(db_pool, &found_quote_id.to_string()).await,
        Err(e) => {
            tracing::warn!("API: Failed to get random quote: {}", e);
            if matches!(e, sqlx::Error::RowNotFound) 
            
            {

                Err(http::StatusCode::NOT_FOUND)
            } else 
            
            {
                Err(http::StatusCode::INTERNAL_SERVER_ERROR)
            }
        }



    }

    
}
