use crate::swagger::SecurityAddon;
use actix_web::web::{scope, ServiceConfig};

use crate::user::create::create as create_user;
use crate::user::delete::delete as delete_user;
use crate::user::read::read as read_user;
use crate::user::update::update as update_user;
use common::management::SearchResult;
use common::user::userable::Userable;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

mod create;
mod delete;
mod read;
mod update;

pub(crate) fn init(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("")
            .service(read_user)
            .service(create_user)
            .service(update_user)
            .service(delete_user),
    );
}

#[derive(Default, Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct User {
    #[serde(default)]
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Default, Serialize, Deserialize, ToSchema)]
pub(super) struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

impl Userable for User {
    fn get_id(&self) -> Box<dyn Display + Sync + Send> {
        Box::new(self.id) as Box<dyn Display + Sync + Send>
    }

    fn get_first_name(&self) -> String {
        self.first_name.to_owned()
    }

    fn get_lastname(&self) -> String {
        self.last_name.to_owned()
    }

    fn get_email(&self) -> String {
        self.email.to_owned()
    }
}

impl From<Box<dyn Userable>> for User {
    fn from(value: Box<dyn Userable>) -> Self {
        Self {
            id: Uuid::from_str(value.get_id().to_string().as_str()).unwrap(),
            first_name: value.get_first_name(),
            last_name: value.get_lastname(),
            email: value.get_email(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserSearchResult {
    pub num_pages: usize,
    pub result: Vec<User>,
}

impl SearchResult for UserSearchResult {
    type Result = User;

    fn get_num_pages(&self) -> usize {
        self.num_pages.to_owned()
    }

    fn get_result(&self) -> Box<dyn Iterator<Item = Self::Result>> {
        Box::new(self.result.clone().into_iter())
    }
}

pub fn api_docs() -> utoipa::openapi::OpenApi {
    #[derive(OpenApi)]
    #[openapi(
    paths(crate::user::create::create,crate::user::read::read,crate::user::update::update,crate::user::delete::delete),
    components(schemas(crate::user::NewUser,crate::user::User)),
    modifiers(&SecurityAddon)
    )]
    struct ApiDocs;
    ApiDocs::openapi()
}

#[cfg(test)]
mod tests {
    use crate::tests::InitializeDB;
    use crate::user::User;
    use common::management::Manageable;
    use futures::future::BoxFuture;
    use futures::FutureExt;
    use log::Level;
    use sea_orm::{ConnectOptions, Database, DatabaseConnection};
    use services_local::user::UserManagement;

    fn initialize_user<'a>(db_url: String, loglevel: Level) -> BoxFuture<'a, ()> {
        async move {
            let mut db_connection_opt = ConnectOptions::new(db_url);
            db_connection_opt.sqlx_logging_level(loglevel.to_level_filter());

            let db_connection: DatabaseConnection = Database::connect(db_connection_opt)
                .await
                .expect("Error occurs when trying to connect to database");

            let new_user = User {
                id: Default::default(),
                first_name: String::from("John"),
                last_name: String::from("Doe"),
                email: String::from("johndoe@example.com"),
            };
            let user_management = UserManagement;
            user_management
                .create(Box::new(new_user), &db_connection)
                .await
                .unwrap();
        }
        .boxed()
    }

    inventory::submit! {
        InitializeDB {
            instruction: initialize_user
        }
    }
}
