use crate::error::AsHttpError;
use crate::user::User;
use crate::{AppState, Management};
use actix_web::web::{Data, Json, Path};
use actix_web::{put, Responder};
use std::sync::Arc;

#[utoipa::path(
tag = "Update user",
context_path = "/api/user",
params(
("user_id" = Uuid, Path, description = "User identifier")
),
request_body = User,
responses((status=200, description = "User update succeed"),
(status=400, description = "Sent data not correct"),
(status=401, description = "Authentication required")
)
)]
#[put("/{user_id}")]
pub(super) async fn update(
    user_id: Path<String>,
    user: Json<User>,
    app_data: Data<Arc<AppState>>,
) -> actix_web::Result<impl Responder> {
    let user_id = user_id.into_inner();
    let management: Management<_, _, _, _, _> =
        Management(Box::new(services_local::user::UserManagement));
    let response = management
        .0
        .update(
            Box::new(user_id),
            Box::new(user.into_inner()),
            &app_data.db_connection,
        )
        .await
        .map_err(|e| AsHttpError::from(e.get_core_error()))?;
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use crate::tests::initialize_db;
    use crate::user::create::create;
    use crate::user::update::update;
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
    async fn test_user_update() {
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
                .service(web::scope("/api/user").service(create).service(update)),
        )
        .await;

        let new_user = NewUser {
            first_name: "Jean".to_string(),
            last_name: "Luc".to_string(),
            email: "jeanluc@example.com".to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/api/user")
            .set_json(&new_user)
            .to_request();
        let resp = test::call_service(&app, req).await;
        let user: User = test::read_body_json(resp).await;
        println!("user id: {:?}", user.id);

        let mut path = String::from("/api/user/");
        path.push_str(&user.id.clone().to_string());

        // test 200
        {
            let mut update_user = user;
            update_user.first_name = String::from("Jane");
            let req = test::TestRequest::put()
                .uri(path.as_str())
                .set_json(&update_user)
                .to_request();
            let resp = test::call_service(&app, req).await;
            println!("status code: {:?}", resp.status().to_string());
            assert_eq!(resp.status(), StatusCode::OK);
            let response = test::read_body(resp).await;
            println!("response body: {:?}", response);
            let updated_user: User = serde_json::from_slice(response.iter().as_slice()).unwrap();
            assert_eq!(updated_user.first_name, update_user.first_name);
            assert_eq!(updated_user.last_name, update_user.last_name);
            assert_eq!(updated_user.email, update_user.email);
            assert_eq!(updated_user.id, update_user.id);
        }

        // test 400
        {
            let req = test::TestRequest::put()
                .uri(path.as_str())
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
