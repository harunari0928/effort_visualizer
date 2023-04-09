use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use mockall::automock;

use crate::domain::users::User;
use crate::dto::{LoginResult, LoginSituation, SignupRequest, SignupResult, SignupSituation};
use crate::{helpers::environments::EnvVariables, repositories::users_repository::UserRepository};

#[automock]
#[async_trait]
pub trait AuthenticationUsecase {
    async fn login(&self, credential: &str) -> Result<LoginResult>;
    async fn signup(&self, request: &SignupRequest) -> Result<SignupResult>;
}

pub struct AuthenticationUsecaseImpl {
    env_variables: Arc<EnvVariables>,
    user_repository: Box<dyn UserRepository + Send + Sync>,
}

impl AuthenticationUsecaseImpl {
    pub fn new(
        env_variables: Arc<EnvVariables>,
        user_repository: Box<dyn UserRepository + Send + Sync>,
    ) -> Self {
        Self {
            env_variables,
            user_repository,
        }
    }

    fn verify_token(&self, credential: &str) -> Result<google_signin::IdInfo> {
        let mut client = google_signin::Client::new();
        client
            .audiences
            .push(self.env_variables.google_client_id.to_owned());
        client
            .verify(credential)
            .context("Token verification failed.")
    }
}

#[async_trait]
impl AuthenticationUsecase for AuthenticationUsecaseImpl {
    async fn login(&self, credential: &str) -> Result<LoginResult> {
        let id_token = match self.verify_token(credential) {
            Ok(id_token) => id_token,
            Err(e) => {
                return Ok(LoginResult {
                    situation: LoginSituation::VerificationFailed,
                    login_user: None,
                    description: Some(e.to_string()),
                })
            }
        };
        let email = match id_token.email {
            Some(email) => email,
            None => {
                return Ok(LoginResult {
                    situation: LoginSituation::EmailIsEmpty,
                    login_user: None,
                    description: Some("Email is empty.".to_string()),
                })
            }
        };
        let user = match self.user_repository.find(&email).await? {
            Some(user) => user,
            None => {
                return Ok(LoginResult {
                    situation: LoginSituation::NotRegistered,
                    login_user: None,
                    description: None,
                })
            }
        };
        Ok(LoginResult {
            situation: LoginSituation::Succeeded,
            login_user: Some(user),
            description: None,
        })
    }

    async fn signup(&self, request: &SignupRequest) -> Result<SignupResult> {
        if request.user_name.is_empty() {
            return Ok(SignupResult {
                situation: SignupSituation::UserNameIsEmpty,
                login_user: None,
                description: None,
            });
        }

        let id_token = match self.verify_token(&request.token.credential) {
            Ok(id_token) => id_token,
            Err(e) => {
                return Ok(SignupResult {
                    situation: SignupSituation::VerificationFailed,
                    login_user: None,
                    description: Some(e.to_string()),
                })
            }
        };

        let email = match id_token.email {
            Some(email) => email,
            None => {
                return Ok(SignupResult {
                    situation: SignupSituation::EmailIsEmpty,
                    login_user: None,
                    description: None,
                })
            }
        };

        if let Some(user) = self.user_repository.find(&email).await? {
            return Ok(SignupResult {
                situation: SignupSituation::AlreadyRegistered,
                login_user: Some(user),
                description: None,
            });
        }

        let new_user = User {
            email,
            external_id: id_token.sub,
            user_name: request.user_name.to_owned(),
            registered_date: std::time::SystemTime::now(),
            updated_date: std::time::SystemTime::now(),
        };
        self.user_repository.add(&new_user).await?;
        Ok(SignupResult {
            situation: SignupSituation::Succeeded,
            login_user: Some(new_user),
            description: None,
        })
    }
}
