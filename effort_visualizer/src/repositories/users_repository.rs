use crate::domain::users::User;
use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;
use tokio_postgres::{types::ToSql, Client, NoTls, Row};
use tracing::error;

#[automock]
#[async_trait]
pub trait UserRepository: Send {
    async fn add(&self, data: &User) -> Result<()>;
    async fn find(&self, email: &str) -> Result<Option<User>>;
}

pub struct UserRepositoryImpl {
    connection_str: String,
}

impl UserRepositoryImpl {
    pub fn new(
        server: String,
        port: String,
        database: String,
        user_id: String,
        password: String,
    ) -> Self {
        let connection_str = String::from(&format!(
            "postgresql://{user_id}:{password}@{server}:{port}/{database}"
        ));
        Self { connection_str }
    }

    async fn connect_database(&self) -> Result<Client> {
        let (client, connection) = tokio_postgres::connect(&self.connection_str, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("connection error: {}", e);
            }
        });
        Ok(client)
    }

    fn parse_query_result(&self, result: Vec<Row>) -> Result<Option<User>> {
        Ok(if result.is_empty() {
            None
        } else {
            Some(User {
                email: result[0].get("email"),
                external_id: result[0].get("external_id"),
                user_name: result[0].get("user_name"),
                registered_date: result[0].get("registered_date"),
                updated_date: result[0].get("updated_date"),
            })
        })
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn add(&self, data: &User) -> Result<()> {
        let row: Vec<&'_ (dyn ToSql + Sync)> = vec![
            &data.email,
            &data.external_id,
            &data.user_name,
            &data.registered_date,
            &data.updated_date,
        ];
        self.connect_database()
            .await?
            .query(
                "
                INSERT INTO users (
                    email,
                    external_id,
                    user_name,
                    registered_date,
                    updated_date)
                VALUES ($1, $2, $3, $4, $5)",
                &row,
            )
            .await?;
        Ok(())
    }

    async fn find(&self, email: &str) -> Result<Option<User>> {
        let row: Vec<&'_ (dyn ToSql + Sync)> = vec![&email];
        let query_result = self
            .connect_database()
            .await?
            .query(
                "
                SELECT 
                    email,
                    external_id,
                    user_name,
                    registered_date,
                    updated_date
                FROM users
                WHERE
                    email = $1",
                &row,
            )
            .await?;
        self.parse_query_result(query_result)
    }
}
