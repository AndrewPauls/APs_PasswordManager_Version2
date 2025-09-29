use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPoolOptions;
use std::net::SocketAddr;
use dotenvy::dotenv;
use hyper::Server;
use axum::http::StatusCode;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    message: String,
    http_code: u16,
    data: Option<T>,
}

#[derive(Clone)]
struct AppState {
    db: sqlx::MySqlPool,
}

#[derive(Debug, Deserialize, Serialize)]
struct AddEntry {
    owner: String,
    name: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    account_owner: Option<String>,
    account_name: Option<String>,
    account_username: Option<String>,
    account_password: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to DB");

    let app_state = AppState { db: db_pool };

    let app = Router::new()
        .route("/add", post(add_entry))
        .route("/entries/:owner", get(get_entries))
        .route("/delete/:owner/:name", delete(delete_entry))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn add_entry(
    State(state): State<AppState>,
    Json(payload): Json<AddEntry>,
) -> (StatusCode, Json<ApiResponse<AddEntry>>) {
    let result = sqlx::query!(
        r#"
        INSERT INTO password_records (account_owner, account_name, account_username, account_password)
        VALUES (?, ?, ?, ?)
        "#,
        payload.owner,
        payload.name,
        payload.username,
        payload.password,
    )
    .execute(&state.db)
    .await;

    let (status, message) = match result {
        Ok(_) => (StatusCode::CREATED, "Record added successfully".to_string()),
        Err(e) => {
            eprintln!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add record".to_string())
        }
    };

    let response = ApiResponse {
        message,
        http_code: status.as_u16(),
        data: match status {
            StatusCode::CREATED => Some(payload),
            _ => None,
        },
    };

    (status, Json(response))
}

async fn get_entries(
    State(state): State<AppState>,
    Path(owner): Path<String>,
) -> (StatusCode, Json<ApiResponse<Vec<Entry>>>) {
    let rows_result = sqlx::query_as!(
        Entry,
        r#"
        SELECT account_owner, account_name, account_username, account_password
        FROM password_records
        WHERE account_owner = ?
        "#,
        owner
    )
    .fetch_all(&state.db)
    .await;

    match rows_result {
        Ok(rows) => {
            let status = StatusCode::OK;
            let response = ApiResponse {
                message: "Entries retrieved successfully".to_string(),
                http_code: status.as_u16(),
                data: Some(rows),
            };
            (status, Json(response))
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            let response = ApiResponse {
                message: "Failed to retrieve entries".to_string(),
                http_code: status.as_u16(),
                data: None,
            };
            (status, Json(response))
        }
    }
}

async fn delete_entry(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let result = sqlx::query!(
        r#"
        DELETE FROM password_records
        WHERE account_owner = ? AND account_name = ?
        "#,
        owner,
        name,
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => {
            let status = StatusCode::OK;
            let response = ApiResponse {
                message: "Record deleted successfully".to_string(),
                http_code: status.as_u16(),
                data: None,
            };
            (status, Json(response))
        }
        Ok(_) => {
            let status = StatusCode::NOT_FOUND;
            let response = ApiResponse {
                message: "No matching records found.".to_string(),
                http_code: status.as_u16(),
                data: None,
            };
            (status, Json(response))
        }
        Err(e) => {
            eprintln!("Delete error: {:?}", e);
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            let response = ApiResponse {
                message: "Failed to delete record".to_string(),
                http_code: status.as_u16(),
                data: None,
            };
            (status, Json(response))
        }
    }
}

