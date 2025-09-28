use axum::{
    routing::{get, post, delete},
    Router, Json, extract::{State, Path}
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPoolOptions;
use std::net::SocketAddr;
use dotenvy::dotenv;
use hyper::Server;

#[derive(Clone)]
struct AppState {
    db: sqlx::MySqlPool,
}

#[derive(Debug, Deserialize)]
struct AddEntry {
    owner: String,
    name: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct Entry {
    account_owner: Option<String>,
    account_name: Option<String>,
    account_username: Option<String>,
    account_password: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

    let db_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to DB");

    let app_state = AppState { db: db_pool };

    let app = Router::new()
        .route("/add", post(add_entry))
        .route("/entries/:owner", get(get_entries))
        .route("/delete/:owner/:name", delete(delete_entry)) // new delete endpoint
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

async fn add_entry(
    State(state): State<AppState>,
    Json(payload): Json<AddEntry>,
) -> Json<ApiResponse> {
    sqlx::query!(
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
    .await
    .unwrap();

    Json(ApiResponse {
        message: "Record added successfully".into(),
    })
}

async fn get_entries(
    State(state): State<AppState>,
    Path(owner): Path<String>,
) -> Json<Vec<Entry>> {
    let rows = sqlx::query_as!(
        Entry,
        r#"
        SELECT account_owner, account_name, account_username, account_password
        FROM password_records
        WHERE account_owner = ?
        "#,
        owner
    )
    .fetch_all(&state.db)
    .await
    .unwrap();

    Json(rows)
}

async fn delete_entry(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> Json<ApiResponse> {
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
        Ok(res) if res.rows_affected() > 0 => Json(ApiResponse {
            message: "Record deleted successfully".to_string(),
        }),
        Ok(_) => Json(ApiResponse {
            message: "No matching records found.".to_string(),
        }),
        Err(e) => {
            eprintln!("Delete error: {:?}", e);
            Json(ApiResponse {
                message: "Failed to delete record".to_string(),
            })
        }
    }
}
