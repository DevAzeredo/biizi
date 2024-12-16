use axum::response::Json;
use diesel::dsl::insert_into;
use diesel_async::AsyncPgConnection;

use crate::{domain::models::{Company, Employee, JobOpportunity}, infrastructure::schema::*};

pub struct Repository;

impl Repository {
    pub async fn save_employee(
        conn: &mut AsyncPgConnection,
        new_employee: &Employee,
    ) -> Result<Json<Employee>, diesel::result::Error> {
        let res = insert_into(employees::table)
            .values(new_employee)
            .returning(employees::as_returning())
            .get_result(&mut conn)
            .await?;
        Ok(Json(res))
    }

    pub async fn save_company(
        conn: &mut AsyncPgConnection,
        new_company: &Company,
    ) -> Result<(), diesel::result::Error> {
        let res = insert_into(companies::table)
            .values(new_company)
            .returning(companies::as_returning())
            .get_result(&mut conn)
            .await?;
        Ok(Json(res))
    }

    pub async fn save_job_opportunity(
        conn: &mut AsyncPgConnection,
        new_job: &JobOpportunity,
    ) -> Result<(), diesel::result::Error> {
        let res = insert_into(job_opportunities::table)
            .values(new_job)
            .returning(job_opportunities::as_returning())
            .get_result(&mut conn)
            .await?;
        Ok(Json(res))
    }
}