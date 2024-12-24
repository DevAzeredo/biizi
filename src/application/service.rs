use axum::Json;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};

use crate::{
    domain::models::{
        Company, Employee, JobOpportunity, NewCompany, NewEmployee, NewJobOpportunity, NewUser,
        User,
    },
    infrastructure::{auth::Auth, repositories::Repository},
};

pub struct Service;
impl Service {
    pub async fn find_by_login(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        user_login: &str,
    ) -> Result<User, diesel::result::Error> {
        Repository::find_by_login(conn, user_login).await
    }

    pub async fn find_employee(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        employe_id: &i64,
    ) -> Result<Employee, diesel::result::Error> {
        Repository::find_employe(conn, employe_id).await
    }
    
    pub async fn find_company(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
       company_id: &i64,
    ) -> Result<Company, diesel::result::Error> {
        Repository::find_company(conn, company_id).await
    }


    pub async fn add_employee(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        employee: NewEmployee,
        user: User,
    ) -> Result<Json<Employee>, diesel::result::Error> {
            Repository::save_employee(conn, &employee, &user).await
    }

    pub async fn add_company(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        company: NewCompany,
        user: User,
    ) -> Result<Json<Company>, diesel::result::Error> {
        Repository::save_company(conn, &company, &user).await
    }

    pub async fn add_job_opportunity(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        job: NewJobOpportunity,
        user: User,
    ) -> Result<Json<JobOpportunity>, diesel::result::Error> {
        println!("{:?}", job);
        println!("{:?}", user);
        let new_job = NewJobOpportunity {
            company_id: user.companyid,
            ..job.clone()
        };

        Repository::save_job_opportunity(conn, &new_job).await
    }

    pub async fn register_user(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        new_user: NewUser,
    ) -> Result<String, diesel::result::Error> {
        match Repository::find_by_login(conn, &new_user.login).await {
            Ok(_) => return Err(diesel::result::Error::BrokenTransactionManager),
            Err(_) => {}
        }

       let hashed_password = match Auth::hash_password(&new_user.password.clone()) {
            Ok(hashed_pass) => hashed_pass,
            Err(_) => return Err(diesel::result::Error::BrokenTransactionManager),
        };

        let new_user_hashed = NewUser{
            login : new_user.login.clone(),
            password : hashed_password
        };

        let user = Repository::save_user(conn, &new_user_hashed).await?;
        let token = match Auth::encode_jwt(user.login.clone()) {
            Ok(token) => Ok(token),
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        };
        Ok(token?)
    }
}
