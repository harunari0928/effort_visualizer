use super::errors::ApiError;
use crate::dto::{LoginRequest, LoginSituation, SignupRequest, SignupSituation};
use crate::usecases::authentication_usecase::AuthenticationUsecase;
use actix_session::Session;
use actix_web::error::ErrorInternalServerError;
use actix_web::{
    post,
    web::{self, Data},
    HttpResponse,
};
use anyhow::Result;
use futures::TryFutureExt;

#[utoipa::path(
    post,
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login user", body = LoginResult),
        (status = 202, description = "Not Registered", body = LoginResult),
        (status = 401, description = "Login failed", body = LoginResult),
        (status = 500, description = "Internal error")
    ),
)]
#[post("/login")]
pub async fn login(
    session: Session,
    usecase: Data<Box<dyn AuthenticationUsecase>>,
    credential_info: web::Json<LoginRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let result = usecase
        .login(&credential_info.credential)
        .map_err(ApiError::from)
        .await?;
    match result.situation {
        LoginSituation::Succeeded => {
            match &result.login_user {
                Some(user) => session.insert("current_user", user)?,
                None => return Err(ErrorInternalServerError("Missing user")),
            }
            Ok(HttpResponse::Ok().json(result))
        }
        LoginSituation::NotRegistered => Ok(HttpResponse::Accepted().json(result)),
        _ => Ok(HttpResponse::Unauthorized().json(result)),
    }
}

#[utoipa::path(
    post,
    request_body = SignupRequest,
    responses(
        (status = 200, description = "Sign up is succeeded.", body = SignupResult),
        (status = 202, description = "The user is already registered.", body = SignupResult),
        (status = 401, description = "Login failed.", body = SignupResult),
        (status = 500, description = "Internal error.")
    ),
)]
#[post("/signup")]
pub async fn signup(
    session: Session,
    usecase: Data<Box<dyn AuthenticationUsecase>>,
    signup_info: web::Json<SignupRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let result = usecase.signup(&signup_info).map_err(ApiError::from).await?;
    match result.situation {
        SignupSituation::Succeeded => {
            match &result.login_user {
                Some(user) => session.insert("current_user", user)?,
                None => return Err(ErrorInternalServerError("Missing user")),
            }
            Ok(HttpResponse::Ok().json(result))
        }
        SignupSituation::AlreadyRegistered => Ok(HttpResponse::Accepted().json(result)),
        _ => Ok(HttpResponse::Unauthorized().json(result)),
    }
}
