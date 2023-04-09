use utoipa::OpenApi;

use super::super::dto::{
    LoginRequest, LoginResult, LoginSituation, SignupRequest, SignupResult, SignupSituation,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controllers::authentication_controllers::login,
        crate::controllers::authentication_controllers::signup
    ),
    components(schemas(
        LoginRequest,
        LoginResult,
        SignupRequest,
        SignupResult,
        LoginSituation,
        SignupSituation
    ))
)]
pub struct ApiDoc;
