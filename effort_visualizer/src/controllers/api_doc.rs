use utoipa::OpenApi;

use super::authentication_controllers::{
    LoginInfo, LoginResult, LoginSituation, SignupInfo, SignupResult, SignupSituation,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controllers::authentication_controllers::login,
        crate::controllers::authentication_controllers::signup
    ),
    components(schemas(
        LoginInfo,
        LoginResult,
        SignupInfo,
        SignupResult,
        LoginSituation,
        SignupSituation
    ))
)]
pub struct ApiDoc;
