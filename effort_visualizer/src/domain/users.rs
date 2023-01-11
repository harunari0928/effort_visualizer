use chrono::NaiveDate;

pub struct User {
    pub email: String,
    pub external_id: String,
    pub user_name: String,
    pub registered_date: NaiveDate,
    pub updated_date: NaiveDate,
}
