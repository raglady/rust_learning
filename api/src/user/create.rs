use crate::error::AsHttpError;
use crate::user::User;
use crate::{AppState, Management};
use actix_web::web::{Data, Json};
use actix_web::{post, HttpResponse, Responder};
use std::sync::Arc;

#[utoipa::path(
tag = "Create user",
context_path = "/api/user",
request_body = NewUser,
responses((status=201, description = "User creation succeed"),
(status=400, description = "Data sent not correct"),
(status=401, description = "Authentication required")
)
)]
#[post("")]
pub(super) async fn create(
    user: Json<User>,
    app_data: Data<Arc<AppState>>,
) -> actix_web::Result<impl Responder> {
    let management: Management<_, _, _, _, _> =
        Management(Box::new(services_local::user::UserManagement));
    let response = management
        .0
        .create(Box::new(user.into_inner()), &app_data.db_connection)
        .await
        .map_err(|e| AsHttpError::from(e.get_core_error()))?;
    Ok(HttpResponse::Created().json(response))
}

#[cfg(test)]
mod tests {
    use crate::tests::initialize_db;
    use crate::user::create::create;
    use crate::user::{NewUser, User};
    use crate::AppState;
    use actix_web::http::StatusCode;
    use actix_web::web::Data;
    use actix_web::{test, web, App};
    use sea_orm::{ConnectOptions, Database, DatabaseConnection};
    use serde_json::json;
    use service_config::Settings;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_user_create() {
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
                .service(web::scope("/api/user").service(create)),
        )
        .await;

        // test 201
        {
            let new_user = NewUser {
                first_name: "Jules".to_string(),
                last_name: "RAKOTOBE".to_string(),
                email: "jules.rak@example.com".to_string(),
            };
            let req = test::TestRequest::post()
                .uri("/api/user")
                // .insert_header(ContentType::json())
                .set_json(&new_user)
                .to_request();
            let resp = test::call_service(&app, req).await;
            println!("status code: {:?}", resp.status().to_string());
            assert_eq!(resp.status(), StatusCode::CREATED);
            let response = test::read_body(resp).await;
            println!("response body: {:?}", response);
            let user: User = serde_json::from_slice(response.iter().as_slice()).unwrap();
            assert_eq!(user.first_name, new_user.first_name);
            assert_eq!(user.last_name, new_user.last_name);
            assert_eq!(user.email, new_user.email);
        }

        // test 400
        {
            let req = test::TestRequest::post()
                .uri("/api/user")
                // .insert_header(ContentType::json())
                .set_json(json!({
                    "foo": "bar"
                }))
                .to_request();
            let resp = test::call_service(&app, req).await;
            println!("status code: {:?}", resp.status().to_string());
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
            let response = test::read_body(resp).await;
            println!("response body: {:?}", response);
        }
    }
}
