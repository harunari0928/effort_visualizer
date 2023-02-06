mod controllers;
mod domain;
mod helpers;
mod repositories;

use actix_cors::Cors;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, http, middleware::Logger, web::Data, App, HttpServer};
use anyhow::Result;

use controllers::{
    api_doc::ApiDoc,
    authentication_controllers::{login, signup},
};
use helpers::environments::EnvVariables;
use repositories::users_repository::{UserRepository, UserRepositoryImpl};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use std::env;
use tracing::Level;

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
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/opanapi.json", ApiDoc::openapi()),
            )
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

pub fn get_env_settings() -> Result<EnvVariables> {
    Ok(EnvVariables {
        db_server: env::var("DB_SERVERNAME")?,
        db_port: env::var("DB_PORT")?,
        db_name: env::var("DB_NAME")?,
        db_user_id: env::var("DB_USERID")?,
        db_password: env::var("DB_PASSWORD")?,
        google_client_id: env::var("GOOGLE_CLIENT_ID")?,
    })
}
