use crate::infrastructure::schema::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = employees)]
pub struct Employee {
    pub id: i64,
    pub full_name: String,
    pub date_of_birth: String,
    pub gender: String,
    pub email: String,
    pub phone: String,
    pub residential_address: String,
    pub is_available: bool,
    pub latitude: f64,
    pub longitude: f64,
    pub rating: f64,
}

#[derive(Deserialize, Insertable, Queryable, Clone)]
#[diesel(table_name = employees)]
pub struct NewEmployee {
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

#[derive(Deserialize, Serialize, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = companies)]
pub struct Company {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub address: String,
    pub logo_url: String,
}

#[derive(Deserialize, Insertable, Queryable, Clone)]
#[diesel(table_name = companies)]
pub struct NewCompany {
    pub name: String,
    pub description: String,
    pub address: String,
    pub logo_url: Option<String>,
}

#[derive(Deserialize, Serialize, Queryable, Selectable, Identifiable, Clone, AsChangeset)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i64,
    pub login: String,
    pub password: String,
    pub companyid: Option<i64>,
    pub employeeid: Option<i64>
}

#[derive(Deserialize, Insertable, Queryable, Clone)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub login: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = job_opportunities)]
pub struct JobOpportunity {
    pub id: i64,
    pub user_id:Option<i64>,
    pub title: String,
    pub description: String,
    pub company_name: String,
    pub company_logo_url: String,
    pub category: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub start_date_time: String,
    pub duration_in_hours: i32,
    pub pay_rate: f64,
    pub status: String,
}


#[derive(Deserialize, Insertable, Queryable, Clone)]
#[diesel(table_name = job_opportunities)]
pub struct NewJobOpportunity {
    pub title: String,
    pub userId:i64,
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