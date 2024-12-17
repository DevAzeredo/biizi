use axum::Json;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};

use crate::{domain::models::{Company, Employee, JobOpportunity}, infrastructure::repositories::Repository};

pub struct Service;

impl Service {
    pub async fn add_employee(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        employee: Employee,
    ) -> Result<Json<Employee>, diesel::result::Error> {
        Repository::save_employee(conn, &employee).await
    }
    
    pub async fn add_company(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        company: Company,
    ) ->  Result<Json<Company>, diesel::result::Error> {
        Repository::save_company(conn, &company).await
    }

    pub async fn add_job_opportunity(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        job: JobOpportunity,
    ) -> Result<Json<JobOpportunity>, diesel::result::Error> {
        Repository::save_job_opportunity(conn, &job).await
    }
}


