use utoipa::OpenApi;

use super::authentication_controllers::{LoginInfo, SignupInfo};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controllers::authentication_controllers::login,
        crate::controllers::authentication_controllers::signup
    ),
    components(schemas(LoginInfo, SignupInfo))
)]
pub struct ApiDoc;
