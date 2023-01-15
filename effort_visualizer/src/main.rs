pub mod domain;
mod repositories;

use actix_cors::Cors;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key,
    http,
    middleware::Logger,
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::FutureExt;
use repositories::users_repository::{UserRepository, UserRepositoryImpl};
use serde::Deserialize;
use std::env;
use tracing::{error, Level};

use crate::domain::users::User;

#[derive(Deserialize)]
pub struct CredentialInfo {
    credential: String,
}

#[post("/login")]
pub async fn login(
    session: Session,
    env_variables: Data<EnvVariables>,
    user_repository: Data<Box<dyn UserRepository>>,
    credential_info: web::Json<CredentialInfo>,
) -> impl Responder {
    let mut client = google_signin::Client::new();
    client
        .audiences
        .push(env_variables.google_client_id.to_owned());
    let maybe_id_info = client.verify(&credential_info.credential);
    match maybe_id_info {
        Ok(id_token) => match id_token.email {
            None => HttpResponse::Unauthorized().finish(),
            Some(email) => match user_repository.find(&email).await {
                Ok(maybe_user) => match maybe_user {
                    Some(user) => {
                        session.insert("current_user", user);
                        HttpResponse::Ok().finish()
                    }
                    None => HttpResponse::Accepted().finish(),
                },
                Err(error) => HttpResponse::Unauthorized().body(error.to_string()),
            },
        },
        Err(e) => HttpResponse::Forbidden().body(e.to_string()),
    }
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
) -> impl Responder {
    if signup_info.user_name.is_empty() {
        HttpResponse::BadRequest().body("user_name is empty.")
    } else {
        let mut client = google_signin::Client::new();
        client
            .audiences
            .push(env_variables.google_client_id.to_owned());
        let maybe_id_info = client.verify(&signup_info.token.credential);
        match maybe_id_info {
            Ok(id_token) => match id_token.email {
                None => HttpResponse::Unauthorized().finish(),
                Some(email) => match user_repository.find(&email).await {
                    Ok(maybe_user) => match maybe_user {
                        Some(_) => HttpResponse::Forbidden().body("you already sign up."),
                        None => {
                            let new_user = User {
                                email,
                                external_id: id_token.sub,
                                user_name: signup_info.user_name.to_owned(),
                                registered_date: std::time::SystemTime::now(),
                                updated_date: std::time::SystemTime::now(),
                            };
                            match user_repository
                                .add(new_user.clone())
                                .await
                                .context("user insert is failed.")
                            {
                                Ok(_) => {
                                    session.insert("current_user", new_user);
                                    HttpResponse::Ok().finish()
                                }
                                Err(e) => {
                                    error!("{}", e.to_string());
                                    HttpResponse::InternalServerError().finish()
                                }
                            }
                        }
                    },
                    Err(error) => HttpResponse::Unauthorized().body(error.to_string()),
                },
            },
            Err(e) => HttpResponse::Unauthorized().body(e.to_string()),
        }
    }
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
