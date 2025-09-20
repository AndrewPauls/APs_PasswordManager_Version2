use axum::{
    routing::{get, post},
    Router, Json, extract::State
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
    let db_url = "mysql://appuser:kZMHz43s3D8!!@localhost:3306/password_manager";

    let db_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to DB");

    let app_state = AppState { db: db_pool };

    let app = Router::new()
        .route("/add", post(add_entry))
        .route("/entries/:owner", get(get_entries))
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
) -> Json<&'static str> {
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

    Json("Record added successfully")
}

async fn get_entries(
    State(state): State<AppState>,
    axum::extract::Path(owner): axum::extract::Path<String>,
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
