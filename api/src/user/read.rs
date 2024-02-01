use crate::error::AsHttpError;
use crate::user::{User, UserSearchResult};
use crate::{AppState, Management};
use actix_web::web::{Data, Json};
use actix_web::{get, web, Responder};
use chrono::{DateTime, Utc};
use common::management::Searchable;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Default, Serialize, Deserialize)]
struct QuerySearch {
    id: Option<Uuid>,
    pattern: Option<String>,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
    page: Option<usize>,
    per_page: Option<usize>,
}

impl Searchable for QuerySearch {
    type Id = Box<dyn Display + Sync + Send>;

    fn get_id(&self) -> Option<Self::Id> {
        self.id.map(|x| Box::new(x) as Self::Id)
    }

    fn get_pattern(&self) -> Option<Box<dyn Display + Sync + Send>> {
        self.pattern
            .clone()
            .map(|value| Box::new(value) as Box<dyn Display + Sync + Send>)
    }

    fn get_date_range(
        &self,
    ) -> Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> {
        if self.start_date.is_some() && self.end_date.is_some() {
            Some((self.start_date.unwrap(), self.end_date.unwrap()))
        } else {
            None
        }
    }

    fn get_page(&self) -> usize {
        self.page.map_or(1, |p| p)
    }
    fn get_per_page(&self) -> usize {
        self.per_page.map_or(25, |p| p)
    }
}

pub(crate) struct Search(Box<dyn Searchable<Id = Box<dyn Display + Sync + Send>>>);

#[utoipa::path(
tag = "List users",
context_path = "/api/user",
responses((status=200, description = "Users list succeed"),
(status=401, description = "Authentication required")
)
)]
#[get("")]
pub(super) async fn read(
    app_data: Data<Arc<AppState>>,
    query: web::Query<QuerySearch>,
) -> actix_web::Result<impl Responder> {
    let (management, search): (Management<_, _, _, _, _>, Search) = (
        Management(Box::new(services_local::user::UserManagement)),
        Search(Box::new(query.into_inner())),
    );
    let response: UserSearchResult = management
        .0
        .read(search.0, &app_data.db_connection)
        .await
        .map(|x| UserSearchResult {
            num_pages: x.get_num_pages(),
            result: x.get_result().map(User::from).collect(),
        })
        .map_err(|e| AsHttpError::from(e.get_core_error()))?;
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use crate::tests::initialize_db;
    use crate::AppState;
    use actix_web::http::StatusCode;
    use actix_web::web::Data;
    use actix_web::{test, web, App};
    use sea_orm::{ConnectOptions, Database, DatabaseConnection};
    use std::sync::Arc;

    use crate::user::read::read;
    use crate::user::UserSearchResult;
    use service_config::Settings;

    #[tokio::test]
    async fn test_user_read() {
        initialize_db().await;
        std::env::set_var("APP_ENVIRONMENT", "test");
        let settings = Settings::load().expect("Error occurs when trying to load settings");

        let database_url = settings.database.get_url();
        let mut db_connection_opt = ConnectOptions::new(database_url.to_owned());
        db_connection_opt.sqlx_logging_level(settings.application.loglevel.to_level_filter());

        let db_connection: DatabaseConnection = Database::connect(db_connection_opt)
            .await
            .expect("Error occurs when trying to connect to database");

        let state = AppState { db_connection };
        let state = Arc::new(state);

        let app = test::init_service(
            App::new()
                .app_data(Data::new(Arc::clone(&state)))
                .service(web::scope("/api/user").service(read)),
        )
        .await;

        // test 200
        {
            let req = test::TestRequest::default().uri("/api/user").to_request();
            let resp = test::call_service(&app, req).await;
            println!("status code: {:?}", resp.status().to_string());
            assert_eq!(resp.status(), StatusCode::OK);
            let response = test::read_body(resp).await;
            println!("response body: {:?}", response);
            let _user: UserSearchResult =
                serde_json::from_slice(response.iter().as_slice()).unwrap();
        }
    }
}
