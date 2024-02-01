use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::Modify;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let security_scheme = SecurityScheme::Http(
            HttpBuilder::new()
                .scheme(HttpAuthScheme::Bearer)
                .bearer_format("JWT")
                .build(),
        );
        let components = match openapi.components.clone() {
            Some(mut components) => {
                components.add_security_scheme("api_jwt_token", security_scheme);
                components
            }
            None => utoipa::openapi::ComponentsBuilder::new()
                .security_scheme("api_jwt_token", security_scheme)
                .build(),
        };
        openapi.components = Some(components);
    }
}
