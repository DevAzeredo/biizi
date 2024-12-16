use diesel_async::AsyncPgConnection;

use crate::{domain::models::{Company, Employee, JobOpportunity}, infrastructure::repositories::Repository};

pub struct Service;

impl Service {
    pub async fn add_employee(
        conn: &mut AsyncPgConnection,
        employee: Employee,
    ) -> Result<(), diesel::result::Error> {
        Repository::save_employee(conn, &employee).await
    }
    
    pub async fn add_company(
        conn: &mut AsyncPgConnection,
        company: Company,
    ) -> Result<(), diesel::result::Error> {
        Repository::save_company(conn, &company).await
    }

    pub async fn add_job_opportunity(
        conn: &mut AsyncPgConnection,
        job: JobOpportunity,
    ) -> Result<(), diesel::result::Error> {
        Repository::save_job_opportunity(conn, &job).await
    }
}


