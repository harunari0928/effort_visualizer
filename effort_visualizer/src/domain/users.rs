use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub email: String,
    pub external_id: String,
    pub user_name: String,
    pub registered_date: std::time::SystemTime,
    pub updated_date: std::time::SystemTime,
}
