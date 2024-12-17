use axum::response::Json;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel::SelectableHelper;
use crate::{domain::models::{Company, Employee, JobOpportunity}, infrastructure::schema::*};

pub struct Repository;

impl Repository {
    pub async fn save_employee(
        conn: &mut AsyncPgConnection,
        new_employee: &Employee,
    ) -> Result<Json<Employee>, diesel::result::Error> {
        let res = diesel::insert_into(employees::table)
            .values(new_employee)
            .returning(Employee::as_returning())
            .get_result(conn)
            .await?;
        Ok(Json(res))
    }

    pub async fn save_company(
        conn: &mut AsyncPgConnection,
        new_company: &Company,
    ) -> Result<Json<Company>, diesel::result::Error> {
        let res = diesel::insert_into(companies::table)
            .values(new_company)
            .returning(Company::as_returning())
            .get_result(conn)
            .await?;
        Ok(Json(res))
    }

    pub async fn save_job_opportunity(
        conn: &mut AsyncPgConnection,
        new_job: &JobOpportunity,
    ) -> Result<Json<JobOpportunity>, diesel::result::Error> {
        let res = diesel::insert_into(job_opportunities::table)
            .values(new_job)
            .returning(JobOpportunity::as_returning())
            .get_result(conn)
            .await?;
        Ok(Json(res))
    }
}