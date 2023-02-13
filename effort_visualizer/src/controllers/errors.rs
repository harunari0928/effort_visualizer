use actix_web::{body::BoxBody, HttpResponse, ResponseError};

use derive_more::{Display, From};

use serde::ser::StdError;
use tracing::log::error;

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
        error!("{}", &self);
        HttpResponse::new(self.status_code())
    }
}
