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

#[cfg(test)]
mod tests {
    mod login {
        use crate::domain::users::User;
        use crate::dto::LoginRequest;
        use crate::dto::LoginResult;
        use crate::dto::LoginSituation;
        use crate::login;
        use crate::usecases::authentication_usecase::{
            AuthenticationUsecase, MockAuthenticationUsecase,
        };
        use actix_web::{body::MessageBody, http, test, web, App};

        #[actix_web::test]
        async fn ログイン成功時ステータス200を返す() {
            let mut mock_usecase = MockAuthenticationUsecase::new();
            mock_usecase.expect_login().returning(|_| {
                Ok(LoginResult {
                    situation: LoginSituation::Succeeded,
                    login_user: Some(User {
                        email: "".to_owned(),
                        external_id: "".to_owned(),
                        user_name: "".to_owned(),
                        registered_date: std::time::SystemTime::now(),
                        updated_date: std::time::SystemTime::now(),
                    }),
                    description: None,
                })
            });
            let usecase = web::Data::new(Box::new(mock_usecase) as Box<dyn AuthenticationUsecase>);

            let app = test::init_service(App::new().app_data(usecase.clone()).service(login)).await;

            let req = test::TestRequest::post()
                .uri("/login")
                .set_json(&LoginRequest {
                    credential: "test".to_owned(),
                })
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(http::StatusCode::OK, resp.status());
        }

        #[actix_web::test]
        async fn ログイン成功時ログイン結果オブジェクトを返す() {
            let mut mock_usecase = MockAuthenticationUsecase::new();
            let login_result = LoginResult {
                situation: LoginSituation::Succeeded,
                login_user: Some(User {
                    email: "".to_owned(),
                    external_id: "".to_owned(),
                    user_name: "".to_owned(),
                    registered_date: std::time::SystemTime::now(),
                    updated_date: std::time::SystemTime::now(),
                }),
                description: None,
            };
            let expected = login_result.clone();
            mock_usecase
                .expect_login()
                .returning(move |_| Ok(login_result.clone()));
            let usecase = web::Data::new(Box::new(mock_usecase) as Box<dyn AuthenticationUsecase>);

            let app = test::init_service(App::new().app_data(usecase.clone()).service(login)).await;

            let req = test::TestRequest::post()
                .uri("/login")
                .set_json(&LoginRequest {
                    credential: "test".to_owned(),
                })
                .to_request();

            let resp = test::call_service(&app, req).await;
            let login_result_from_response: LoginResult =
                serde_json::from_slice(resp.into_body().try_into_bytes().unwrap().as_ref())
                    .unwrap();

            assert_eq!(expected, login_result_from_response);
        }

        #[actix_web::test]
        async fn ログインに成功したがユーザを取得できなかった時ステータス500を返す() {
            let mut mock_usecase = MockAuthenticationUsecase::new();
            mock_usecase.expect_login().returning(|_| {
                Ok(LoginResult {
                    situation: LoginSituation::Succeeded,
                    login_user: None,
                    description: None,
                })
            });
            let usecase = web::Data::new(Box::new(mock_usecase) as Box<dyn AuthenticationUsecase>);

            let app = test::init_service(App::new().app_data(usecase.clone()).service(login)).await;

            let req = test::TestRequest::post()
                .uri("/login")
                .set_json(&LoginRequest {
                    credential: "test".to_owned(),
                })
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(http::StatusCode::INTERNAL_SERVER_ERROR, resp.status());
        }

        #[actix_web::test]
        async fn 会員未登録時ステータス202を返す() {
            let mut mock_usecase = MockAuthenticationUsecase::new();
            mock_usecase.expect_login().returning(|_| {
                Ok(LoginResult {
                    situation: LoginSituation::NotRegistered,
                    login_user: None,
                    description: None,
                })
            });
            let usecase = web::Data::new(Box::new(mock_usecase) as Box<dyn AuthenticationUsecase>);

            let app = test::init_service(App::new().app_data(usecase.clone()).service(login)).await;

            let req = test::TestRequest::post()
                .uri("/login")
                .set_json(&LoginRequest {
                    credential: "test".to_owned(),
                })
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(http::StatusCode::ACCEPTED, resp.status());
        }

        #[actix_web::test]
        async fn 会員未登録時ログイン結果オブジェクトを返す() {
            let mut mock_usecase = MockAuthenticationUsecase::new();
            let login_result = LoginResult {
                situation: LoginSituation::NotRegistered,
                login_user: None,
                description: None,
            };
            let expected = login_result.clone();
            mock_usecase
                .expect_login()
                .returning(move |_| Ok(login_result.clone()));
            let usecase = web::Data::new(Box::new(mock_usecase) as Box<dyn AuthenticationUsecase>);

            let app = test::init_service(App::new().app_data(usecase.clone()).service(login)).await;

            let req = test::TestRequest::post()
                .uri("/login")
                .set_json(&LoginRequest {
                    credential: "test".to_owned(),
                })
                .to_request();

            let resp = test::call_service(&app, req).await;
            let login_result_from_response: LoginResult =
                serde_json::from_slice(resp.into_body().try_into_bytes().unwrap().as_ref())
                    .unwrap();

            assert_eq!(expected, login_result_from_response);
        }

        #[actix_web::test]
        async fn emailが空のとき時ステータス401を返す() {
            let mut mock_usecase = MockAuthenticationUsecase::new();
            mock_usecase.expect_login().returning(|_| {
                Ok(LoginResult {
                    situation: LoginSituation::EmailIsEmpty,
                    login_user: None,
                    description: None,
                })
            });
            let usecase = web::Data::new(Box::new(mock_usecase) as Box<dyn AuthenticationUsecase>);

            let app = test::init_service(App::new().app_data(usecase.clone()).service(login)).await;

            let req = test::TestRequest::post()
                .uri("/login")
                .set_json(&LoginRequest {
                    credential: "test".to_owned(),
                })
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(http::StatusCode::UNAUTHORIZED, resp.status());
        }

        #[actix_web::test]
        async fn emailが空のとき時結果オブジェクトを返す() {
            let mut mock_usecase = MockAuthenticationUsecase::new();
            let login_result = LoginResult {
                situation: LoginSituation::EmailIsEmpty,
                login_user: None,
                description: None,
            };
            let expected = login_result.clone();
            mock_usecase
                .expect_login()
                .returning(move |_| Ok(login_result.clone()));
            let usecase = web::Data::new(Box::new(mock_usecase) as Box<dyn AuthenticationUsecase>);

            let app = test::init_service(App::new().app_data(usecase.clone()).service(login)).await;

            let req = test::TestRequest::post()
                .uri("/login")
                .set_json(&LoginRequest {
                    credential: "test".to_owned(),
                })
                .to_request();

            let resp = test::call_service(&app, req).await;
            let login_result_from_response: LoginResult =
                serde_json::from_slice(resp.into_body().try_into_bytes().unwrap().as_ref())
                    .unwrap();

            assert_eq!(expected, login_result_from_response);
        }
    }
}
