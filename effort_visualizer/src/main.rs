pub mod domain;
mod repositories;

use crate::domain::users::User;
use actix_cors::Cors;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    body::BoxBody,
    cookie::Key,
    http,
    middleware::Logger,
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, ResponseError,
};
use anyhow::{Context, Result};
use derive_more::{Display, From};
use repositories::users_repository::{UserRepository, UserRepositoryImpl};
use serde::ser::StdError;
use serde::Deserialize;

use std::env;
use tracing::Level;

#[derive(Debug, Display, From)]
#[display(fmt = "{}", _0)]
pub struct ApiError(anyhow::Error);

impl StdError for ApiError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.0.as_ref())
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::new(self.status_code())
    }
}

#[derive(Deserialize)]
pub struct CredentialInfo {
    credential: String,
}

fn verify_token(
    env_variables: &EnvVariables,
    credential: &String,
) -> Result<google_signin::IdInfo> {
    let mut client = google_signin::Client::new();
    client
        .audiences
        .push(env_variables.google_client_id.to_owned());
    client
        .verify(credential)
        .context("Token verification failed.")
}

async fn find_user(
    user_repository: &Data<Box<dyn UserRepository>>,
    email: &str,
) -> Result<Option<User>, ApiError> {
    user_repository.find(email).await.map_err(ApiError::from)
}

async fn add_user(
    user_repository: &Data<Box<dyn UserRepository>>,
    data: &User,
) -> Result<(), ApiError> {
    user_repository.add(data).await.map_err(ApiError::from)
}

#[post("/login")]
pub async fn login(
    session: Session,
    env_variables: Data<EnvVariables>,
    user_repository: Data<Box<dyn UserRepository>>,
    credential_info: web::Json<CredentialInfo>,
) -> Result<HttpResponse, actix_web::Error> {
    let id_token = match verify_token(&env_variables, &credential_info.credential) {
        Ok(id_token) => id_token,
        Err(e) => return Ok(HttpResponse::Unauthorized().body(e.to_string())),
    };
    let email = match id_token.email {
        Some(email) => email,
        None => return Ok(HttpResponse::Unauthorized().body("Email is empty.")),
    };
    let user = match find_user(&user_repository, &email).await? {
        Some(user) => user,
        None => return Ok(HttpResponse::Unauthorized().finish()),
    };
    session.insert("current_user", user)?;
    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
pub struct SignupInfo {
    token: CredentialInfo,
    user_name: String,
}

#[post("/signup")]
pub async fn signup(
    session: Session,
    env_variables: Data<EnvVariables>,
    user_repository: Data<Box<dyn UserRepository>>,
    signup_info: web::Json<SignupInfo>,
) -> Result<HttpResponse, actix_web::Error> {
    if signup_info.user_name.is_empty() {
        return Ok(HttpResponse::BadRequest().body("user_name is empty."));
    }

    let id_token = match verify_token(&env_variables, &signup_info.token.credential) {
        Ok(id_token) => id_token,
        Err(e) => return Ok(HttpResponse::Unauthorized().body(e.to_string())),
    };
    let email = match id_token.email {
        Some(email) => email,
        None => return Ok(HttpResponse::Unauthorized().body("Email is empty.")),
    };

    if find_user(&user_repository, &email).await?.is_some() {
        return Ok(HttpResponse::BadRequest().body("You have already signed up."));
    }

    let new_user = User {
        email,
        external_id: id_token.sub,
        user_name: signup_info.user_name.to_owned(),
        registered_date: std::time::SystemTime::now(),
        updated_date: std::time::SystemTime::now(),
    };
    add_user(&user_repository, &new_user).await?;
    session.insert("current_user", new_user)?;
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> Result<()> {
    init_logger();
    std::env::set_var("RUST_LOG", "actix_web=info");
    let secret_key = Key::generate();
    let env = Data::new(get_env_settings()?);
    HttpServer::new(move || {
        let repository: Data<Box<dyn UserRepository>> =
            Data::new(Box::new(UserRepositoryImpl::new(
                env.db_server.to_owned(),
                env.db_port.to_owned(),
                env.db_name.to_owned(),
                env.db_user_id.to_owned(),
                env.db_password.to_owned(),
            )));
        let cors = Cors::default()
            .allowed_origin("http://localhost:8081")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .app_data(env.clone())
            .app_data(repository.clone())
            .service(login)
            .service(signup)
    })
    .bind(("0.0.0.0", 8080))
    .expect("Can't running HTTP Server")
    .run()
    .await?;
    Ok(())
}

fn init_logger() {
    tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::INFO)
        .json()
        .flatten_event(true)
        // build but do not install the subscriber.
        .init();
}

pub struct EnvVariables {
    db_server: String,
    db_port: String,
    db_name: String,
    db_user_id: String,
    db_password: String,
    google_client_id: String,
}

fn get_env_settings() -> Result<EnvVariables> {
    Ok(EnvVariables {
        db_server: env::var("DB_SERVERNAME")?,
        db_port: env::var("DB_PORT")?,
        db_name: env::var("DB_NAME")?,
        db_user_id: env::var("DB_USERID")?,
        db_password: env::var("DB_PASSWORD")?,
        google_client_id: env::var("GOOGLE_CLIENT_ID")?,
    })
}
