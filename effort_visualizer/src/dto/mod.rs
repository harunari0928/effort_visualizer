use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::users::User;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    pub credential: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct LoginResult {
    pub situation: LoginSituation,
    pub login_user: Option<User>,
    pub description: Option<String>,
}

#[derive(Clone,Debug,  Deserialize, PartialEq, Serialize, ToSchema)]
pub enum LoginSituation {
    Succeeded,
    NotRegistered,
    VerificationFailed,
    EmailIsEmpty,
}

#[derive(Deserialize, ToSchema)]
pub struct SignupRequest {
    pub token: LoginRequest,
    pub user_name: String,
}

#[derive(Serialize, ToSchema)]
pub struct SignupResult {
    pub situation: SignupSituation,
    pub login_user: Option<User>,
    pub description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub enum SignupSituation {
    Succeeded,
    AlreadyRegistered,
    VerificationFailed,
    EmailIsEmpty,
    UserNameIsEmpty,
}
