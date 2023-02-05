use actix_web::{body::BoxBody, HttpResponse, ResponseError};

use derive_more::{Display, From};

use serde::ser::StdError;

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
