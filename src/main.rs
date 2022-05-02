use actix_files::NamedFile;
use actix_web::{get, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use log::info;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, Row, SqlitePool};
use std::env;
use std::fs;

static DB_CONN: OnceCell<SqlitePool> = OnceCell::new();
static TOKEN: OnceCell<String> = OnceCell::new();

async fn db_conn() -> Result<&'static SqlitePool, String> {
    DB_CONN
        .get()
        .ok_or("Database not connected yet.".to_string())
}

pub async fn setup_db() -> Result<(), String> {
    let db_location = "db/track.db";

    fs::create_dir_all("db/").expect("Unable to create db directory");
    if !sqlx::Sqlite::database_exists(&db_location)
        .await
        .map_err(|e| format!("Error checking for db: {}", e))?
    {
        sqlx::Sqlite::create_database(&db_location)
            .await
            .map_err(|e| format!("Error creating db: {}", e))?;
    }
    DB_CONN
        .set(
            SqlitePool::connect(&db_location)
                .await
                .map_err(|e| format!("Error connectiong to DB: {}", e))?,
        )
        .map_err(|_| "Error setting database connection".to_string())?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS locations (
            lat TEXT NOT NULL,
            lon TEXT NOT NULL,
            timestamp TEXT NOT NULL UNIQUE,
            hdop TEXT NOT NULL,
            altitude TEXT NOT NULL,
            speed TEXT NOT NULL,
            bearing TEXT NOT NULL
        );",
    )
    .execute(db_conn().await?)
    .await
    .map_err(|e| format!("Error executing query: {}", e))?;

    return Ok(());
}

#[get("/")]
async fn home(_req: HttpRequest) -> Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Location {
    token: String,
    lat: String,
    lon: String,
    timestamp: String,
    hdop: String,
    altitude: String,
    speed: String,
    bearing: String,
}

#[get("/log")]
async fn log_point(_req: HttpRequest, info: web::Query<Location>) -> HttpResponse {
    if &info.token != TOKEN.get().expect("No Token Supplied") {
        return HttpResponse::Unauthorized().body("ERROR: Invalid Token");
    }
    let conn = match db_conn().await {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().body("ERROR"),
    };
    info!("Recording point.");
    match sqlx::query("INSERT INTO locations (lat, lon, timestamp, hdop, altitude, speed, bearing) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&info.lat)
        .bind(&info.lon)
        .bind(&info.timestamp)
        .bind(&info.hdop)
        .bind(&info.altitude)
        .bind(&info.speed)
        .bind(&info.bearing)
        .execute(conn)
        .await
        .map_err(|e| format!("Error executing query: {}", e)) {
        Ok(_) => HttpResponse::Created().body("OK"),
        Err(_) => return HttpResponse::Ok().body("ERROR"),
        }
}

#[get("/locations")]
async fn locations(_req: HttpRequest) -> HttpResponse {
    let conn = match db_conn().await {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().body("ERROR"),
    };
    let points: Vec<Location> = match sqlx::query("SELECT * FROM locations ORDER BY timestamp ASC")
        .fetch_all(conn)
        .await
        .map_err(|e| format!("Error executing query: {}", e))
    {
        Ok(rows) => rows
            .iter()
            .map(|r| Location {
                token: "".to_string(),
                lat: r.get(0),
                lon: r.get(1),
                timestamp: r.get(2),
                hdop: r.get(3),
                altitude: r.get(4),
                speed: r.get(5),
                bearing: r.get(6),
            })
            .collect(),
        Err(_) => return HttpResponse::InternalServerError().body("ERROR"),
    };
    HttpResponse::Ok().json(points)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Token {
    token: String,
}

#[get("/reset")]
async fn reset(_req: HttpRequest, info: web::Query<Token>) -> HttpResponse {
    if &info.token != TOKEN.get().expect("No Token Supplied") {
        return HttpResponse::Unauthorized().body("ERROR: Invalid Token");
    }
    let conn = match db_conn().await {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().body("ERROR"),
    };
    match sqlx::query("DELETE FROM locations")
        .execute(conn)
        .await
        .map_err(|e| format!("Error executing query: {}", e))
    {
        Ok(_) => HttpResponse::Ok().body("OK"),
        Err(_) => HttpResponse::InternalServerError().body("ERROR"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Setting up database.");
    setup_db().await.expect("Unable to setup database.");
    info!("Database setup complete.");
    info!("Database setup complete.");
    TOKEN
        .set(env::var("TOKEN").expect("No token supplied."))
        .expect("Error setting token.");
    println!("Starting http server: 0.0.0.0:8080");
    std::env::set_var("RUST_LOG", "actix_web=info");
    // start http server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
            .service(home)
            .service(log_point)
            .service(locations)
            .service(reset)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
