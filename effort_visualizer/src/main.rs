use std::env;

use actix_cors::Cors;
use actix_web::{http, post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct CredentialInfo {
    credential: String,
}

#[post("/login")]
async fn login(credential_info: web::Json<CredentialInfo>) -> impl Responder {
    let envs = get_env_settings();
    let [_, _, _, _, _, google_client_id] = envs.map(|x| x.unwrap());

    let mut client = google_signin::Client::new();
    client.audiences.push(google_client_id);
    let id_info = client
        .verify(&credential_info.credential)
        .expect("Expected token to be valid");
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8081")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        App::new().wrap(cors).service(login)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

fn get_env_settings() -> [Result<String, String>; 6] {
    [
        "DB_SERVERNAME",
        "DB_USERID",
        "DB_NAME",
        "DB_PORT",
        "DB_PASSWORD",
        "GOOGLE_CLIENT_ID",
    ]
    .map(|key| env::var(key).map_err(|err| format!("{err}({key})")))
}

fn create_error_messages(envs: [Result<String, String>; 6]) -> String {
    envs.iter()
        .filter_map(|x| x.as_ref().err().map(|x| x.to_owned()))
        .collect::<Vec<String>>()
        .join(",\n")
}
