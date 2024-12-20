use crate::{
    domain::models::{
        Company, Employee, JobOpportunity, NewCompany, NewEmployee, NewJobOpportunity, NewUser,
        User,
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
        user: &User,
    ) -> Result<Json<Employee>, diesel::result::Error> {
        let res;
        match user.employeeid.is_some() {
            true => {

                /*pub date_of_birth: String,
                pub gender: String,
                pub email: String,
                pub phone: String,
                pub residential_address: String,
                pub is_available: bool,
                pub latitude: f64,
                pub longitude: f64,
                pub rating: f64,*/

                res = diesel::update(employees::table.find(user.employeeid.unwrap()))
                    .set(full_name:new_employee.full_name)
                    .get_result(conn)
                    .await?;
            }
            false => {
                let new_employee = NewEmployee {
                    rating: 0.0,
                    ..new_employee.clone()
                };

                res = diesel::insert_into(employees::table)
                    .values(new_employee.clone())
                    .get_result(conn)
                    .await?;
            }
        }

        Ok(Json(res))
    }

    pub async fn save_company(
        conn: &mut AsyncPgConnection,
        new_company: &NewCompany,
        user: &User,
    ) -> Result<Json<Company>, diesel::result::Error> {
        let res;

        if user.employeeid.is_some() {
            res = diesel::update(companies::table.find(user.companyId))
                .set(new_company)
                .get_result(conn)
                .await?;
        } else {
            res = diesel::insert_into(companies::table)
                .values(new_company.clone())
                .get_result(conn)
                .await?;
        }

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
        let user = users
            .filter(login.eq(user_login))
            .first::<User>(conn)
            .await?;

        Ok(user)
    }
}
