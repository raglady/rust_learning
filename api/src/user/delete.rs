use crate::error::AsHttpError;
use crate::{AppState, Management};
use actix_web::web::{Data, Path};
use actix_web::{delete, HttpResponse, Responder};
use std::sync::Arc;

#[utoipa::path(
tag = "Delete user",
context_path = "/api/user",
params(
("user_id" = Uuid, Path, description = "User identifier")
),
responses((status=204, description = "User deletion succeed"),
(status=401, description = "Authentication required")
)
)]
#[delete("/{user_id}")]
pub(super) async fn delete(
    user_id: Path<String>,
    app_data: Data<Arc<AppState>>,
) -> actix_web::Result<impl Responder> {
    let user_id = user_id.into_inner();
    let management: Management<_, _, _, _, _> =
        Management(Box::new(services_local::user::UserManagement));
    management
        .0
        .delete(Box::new(user_id), &app_data.db_connection)
        .await
        .map_err(|e| AsHttpError::from(e.get_core_error()))?;
    Ok(HttpResponse::NoContent().finish())
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
    use std::sync::Arc;

    use crate::user::delete::delete;
    use service_config::Settings;

    #[tokio::test]
    async fn test_delete_update() {
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
                .service(web::scope("/api/user").service(create).service(delete)),
        )
        .await;

        let new_user = NewUser {
            first_name: "Luc".to_string(),
            last_name: "Joseph".to_string(),
            email: "lucjoseph@example.com".to_string(),
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

        // test 204
        {
            let req = test::TestRequest::delete().uri(path.as_str()).to_request();
            let resp = test::call_service(&app, req).await;
            println!("status code: {:?}", resp.status().to_string());
            assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        }
    }
}
