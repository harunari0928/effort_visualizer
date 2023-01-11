pub mod domain;
mod repositories;

use actix_cors::Cors;
use actix_web::{
    http,
    middleware::Logger,
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::Result;
use repositories::users_repository::{UserRepository, UserRepositoryImpl};
use serde::Deserialize;
use std::env;
use tracing::Level;

#[derive(Deserialize)]
pub struct CredentialInfo {
    credential: String,
}

#[post("/login")]
pub async fn login(
    env_variables: Data<EnvVariables>,
    user_repository: Data<Box<dyn UserRepository>>,
    credential_info: web::Json<CredentialInfo>,
) -> impl Responder {
    let mut client = google_signin::Client::new();
    client
        .audiences
        .push(env_variables.google_client_id.to_owned());
    let id_info = client
        .verify(&credential_info.credential)
        .expect("Expected token to be valid");
    match id_info.email {
        None => HttpResponse::Unauthorized().finish(),
        Some(email) => match user_repository.find(&email).await {
            Ok(maybe_user) => match maybe_user {
                Some(_) => HttpResponse::Ok().finish(),
                None => HttpResponse::Accepted().finish(),
            },
            Err(error) => HttpResponse::Unauthorized().body(error.to_string()),
        },
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    init_logger();
    std::env::set_var("RUST_LOG", "actix_web=info");
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
            .app_data(env.clone())
            .app_data(repository.clone())
            .service(login)
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
