use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::infrastructure::schema::*;
#[derive(Deserialize, Serialize, Queryable, Insertable)]
#[diesel(table_name = employees)]
pub struct Employee {
    pub id: i64,
    pub full_name: String,
    pub date_of_birth: String,
    pub gender: Option<String>,
    pub email: String,
    pub phone: String,
    pub residential_address: String,
    pub is_available: bool,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub rating: f64,
}

#[derive(Deserialize, Serialize, Queryable, Insertable)]
#[diesel(table_name = companies)]
pub struct Company {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub address: String,
    pub logo_url: String,
}

#[derive(Deserialize, Serialize, Queryable, Insertable)]
#[diesel(table_name = job_opportunities)]
pub struct JobOpportunity {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub company_name: String,
    pub company_logo_url: Option<String>,
    pub category: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub start_date_time: String,
    pub duration_in_hours: i32,
    pub pay_rate: f64,
    pub status: String,
}
