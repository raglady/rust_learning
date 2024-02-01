use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use api::{init, AppState};
use migration::{Migrator, MigratorTrait};
use sea_orm::TransactionTrait;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use service_config::Settings;
use std::sync::Arc;
// use std::thread;
use env_logger::Env;
use log::warn;

pub async fn start() -> std::io::Result<Server> {
    let settings = Settings::load().expect("Error occurs when trying to load settings");
    let _ = env_logger::try_init_from_env(
        Env::new().default_filter_or(settings.application.loglevel.as_str()),
    )
    .map_err(|_err| warn!("Envlogger already inited !"));
    let database_url = settings.database.get_url();
    let mut db_connection_opt = ConnectOptions::new(database_url.to_owned());
    db_connection_opt
        .min_connections(5)
        .max_connections(100)
        .sqlx_logging_level(settings.application.loglevel.to_level_filter());

    let db_connection: DatabaseConnection = Database::connect(db_connection_opt)
        .await
        .expect("Error occurs when trying to connect to database");

    let state = AppState { db_connection };
    let state = Arc::new(state);
    let state_1 = Arc::clone(&state);

    tokio::spawn(async move {
        let transaction_connection = state_1.db_connection.begin().await.unwrap();
        Migrator::up(&transaction_connection, None).await.unwrap();
        transaction_connection.commit().await.unwrap();
    })
    .await
    .expect("Cannot apply migration !");

    let server = HttpServer::new(move || {
        let state = Arc::clone(&state);
        App::new()
            .app_data(Data::new(state))
            .wrap(Logger::default())
            .configure(init)
    })
    .bind(format!(
        "{}:{}",
        settings.application.host.clone(),
        settings.application.port.clone()
    ))?
    .run();
    Ok(server)
}

#[cfg(test)]
mod tests {
    use super::start;
    use tokio::net::TcpStream;
    #[tokio::test]
    async fn test_server() {
        std::env::set_var("APP_APPLICATION__HOST", "127.0.0.1");
        std::env::set_var("APP_APPLICATION__PORT", "8000");
        std::env::set_var("APP_ENVIRONMENT", "test");

        let _handle = start().await.unwrap();

        let mut counter: u8 = 0;
        while let Err(_connection_error) = TcpStream::connect("127.0.0.1:8000").await {
            if counter == 5 {
                panic!("Connection to server failed");
            }
            let _ = tokio::time::sleep(tokio::time::Duration::from_secs(5));
            counter += 1;
        }
    }
}
