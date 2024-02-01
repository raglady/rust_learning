use actix_web::web::{scope, ServiceConfig};

use common::management::Manageable;
use sea_orm::DatabaseConnection;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod error;
pub mod swagger;
mod user;

pub struct AppState {
    pub db_connection: DatabaseConnection,
}

struct Management<'a, B: ?Sized, D: ?Sized, S: ?Sized, R: ?Sized, I: ?Sized>(
    Box<dyn Manageable<'a, B, Data = D, Id = I, Result = R, Search = S>>,
);

pub fn init(cfg: &mut ServiceConfig) {
    #[derive(OpenApi)]
    #[openapi(info(
        title = "Api documentation",
        description = "Api documentation",
        contact(
            name = "Falihery Emile RANDRIANASOLO",
            email = "falihery.randrianasolo@gmail.com"
        )
    ))]
    struct ApiDocs;
    let mut api_docs = ApiDocs::openapi();
    api_docs.merge(user::api_docs());
    cfg.service(scope("/api/user").configure(user::init))
        .service(SwaggerUi::new("/api/docs/{_:.*}").url("/api/api-docs/openapi.json", api_docs));
}

#[cfg(test)]
pub mod tests {
    use futures::future::BoxFuture;
    use inventory;
    use lazy_static::lazy_static;
    use log::Level;
    use migration::{Migrator, MigratorTrait};
    use sea_orm::{ConnectOptions, Database, DatabaseConnection, TransactionTrait};
    use service_config::Settings;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    lazy_static! {
        static ref DB_INITIALIZED: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    }

    pub struct InitializeDB {
        pub instruction: fn(String, Level) -> BoxFuture<'static, ()>,
    }

    inventory::collect!(InitializeDB);

    pub async fn initialize_db() {
        let mut db_initialized = DB_INITIALIZED.lock().await;
        if !(*db_initialized) {
            std::env::set_var("APP_ENVIRONMENT", "test");
            let settings = Settings::load().expect("Error occurs when trying to load settings");

            let database_url = settings.database.get_url();
            let mut db_connection_opt = ConnectOptions::new(database_url.to_owned());
            db_connection_opt.sqlx_logging_level(settings.application.loglevel.to_level_filter());

            let db_connection: DatabaseConnection = Database::connect(db_connection_opt)
                .await
                .expect("Error occurs when trying to connect to database");

            let db_connection: Arc<DatabaseConnection> = Arc::new(db_connection);
            let db_connection_clone = Arc::clone(&db_connection);
            tokio::spawn(async move {
                let transaction_connection = db_connection_clone.begin().await.unwrap();
                Migrator::down(&transaction_connection, None).await.unwrap();
                Migrator::up(&transaction_connection, None).await.unwrap();
                transaction_connection.commit().await.unwrap();
            })
            .await
            .expect("Cannot apply migration !");

            for init in inventory::iter::<InitializeDB> {
                (init.instruction)(
                    database_url.to_owned(),
                    settings.application.loglevel.to_owned(),
                )
                .await;
            }
            *db_initialized = true;
        }
    }
}
