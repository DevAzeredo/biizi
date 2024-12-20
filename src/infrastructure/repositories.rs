use crate::{
    domain::models::{
        Company, Employee, JobOpportunity, NewCompany, NewEmployee, NewJobOpportunity, NewUser, User
    },
    infrastructure::schema::*,
};
use axum::response::Json;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

pub struct Repository;

impl Repository {
    pub async fn save_employee(
        conn: &mut AsyncPgConnection,
        new_employee: &NewEmployee,
    ) -> Result<Json<Employee>, diesel::result::Error> {
        let new_employee = NewEmployee {
            rating: 0.0,
            ..new_employee.clone()
        };
        let res = diesel::insert_into(employees::table)
            .values(new_employee)
            .get_result(conn)
            .await?;

        Ok(Json(res))
    }

    pub async fn save_company(
        conn: &mut AsyncPgConnection,
        new_company: &NewCompany,
    ) -> Result<Json<Company>, diesel::result::Error> {
        let res = diesel::insert_into(companies::table)
            .values(new_company.clone())
            .get_result(conn)
            .await?;

        Ok(Json(res))
    }

    pub async fn save_job_opportunity(
        conn: &mut AsyncPgConnection,
        new_job: &NewJobOpportunity,
    ) -> Result<Json<JobOpportunity>, diesel::result::Error> {
        let res = diesel::insert_into(job_opportunities::table)
            .values(new_job.clone())
            .get_result(conn)
            .await?;
        Ok(Json(res))
    }

    pub async fn save_user(
        conn: &mut AsyncPgConnection,
        new_user: &NewUser,
    ) -> Result<Json<User>, diesel::result::Error> {
        let res = diesel::insert_into(users::table)
            .values(new_user.clone())
            .get_result(conn)
            .await?;
        Ok(Json(res))
    }

    pub async fn find_by_login(
        conn: &mut AsyncPgConnection,
        user_login: &str,
    ) -> Result<User, diesel::result::Error> {
        use crate::infrastructure::schema::users::dsl::*;
        let user = users.filter(login.eq(user_login)).first::<User>(conn).await?;

        Ok(user)
    }
}
