use crate::{
    domain::models::{
        Company, Employee, JobOpportunity, JobOpportunityWithCompany, NewCompany, NewEmployee,
        NewJobOpportunity, NewUser, User,
    },
    infrastructure::schema::*,
};
use axum::response::Json;
use companies::{address, description, logo_url, name};
use diesel::SelectableHelper;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use employees::*;
use users::{companyid, employeeid};

pub struct Repository;

impl Repository {
    pub async fn save_employee(
        conn: &mut AsyncPgConnection,
        new_employee: &NewEmployee,
        user: &User,
    ) -> Result<Json<Employee>, diesel::result::Error> {
        let res: Employee;
        match user.employeeid.is_some() {
            true => {
                res = diesel::update(employees::table.find(user.employeeid.unwrap()))
                    .set((
                        full_name.eq(new_employee.full_name.clone()),
                        gender.eq(new_employee
                            .gender
                            .clone()
                            .unwrap_or_else(|| "undefined".to_string())),
                        email.eq(new_employee.email.clone()),
                        phone.eq(new_employee.phone.clone()),
                        is_available.eq(new_employee.is_available),
                        residential_address.eq(new_employee.residential_address.clone()),
                        latitude.eq(new_employee.latitude.unwrap_or_else(|| 0.0)),
                        longitude.eq(new_employee.longitude.unwrap_or_else(|| 0.0)),
                        date_of_birth.eq(new_employee.date_of_birth.clone()),
                    ))
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

                diesel::update(users::table.filter(users::id.eq(user.id)))
                    .set(employeeid.eq(res.id))
                    .execute(conn)
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
        let res: Company;

        if user.companyid.is_some() {
            res = diesel::update(companies::table.find(user.companyid.unwrap()))
                .set((
                    name.eq(new_company.name.clone()),
                    description.eq(new_company.description.clone()),
                    address.eq(new_company.address.clone()),
                    logo_url.clone().eq(new_company
                        .logo_url
                        .clone()
                        .unwrap_or_else(|| "".to_string())),
                ))
                .get_result(conn)
                .await?;
        } else {
            res = diesel::insert_into(companies::table)
                .values(new_company.clone())
                .get_result(conn)
                .await?;

            diesel::update(users::table.filter(users::id.eq(user.id)))
                .set(companyid.eq(res.id))
                .execute(conn)
                .await?;
        }

        Ok(Json(res))
    }

    pub async fn update_company_logo(
        conn: &mut AsyncPgConnection,
        company_id: &i64,
        logo: &String,
    ) -> Result<Json<Company>, diesel::result::Error> {
        let res = diesel::update(companies::table.find(company_id))
            .set(logo_url.eq(logo))
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

    pub async fn find_job_opportunities_with_company(
        conn: &mut AsyncPgConnection,
        pcompany_id: &i64,
    ) -> Result<Vec<JobOpportunityWithCompany>, diesel::result::Error> {
        use crate::infrastructure::schema::job_opportunities::dsl::*;

        let jobs = job_opportunities
            .filter(company_id.eq(*pcompany_id))
            .select(JobOpportunity::as_select())
            .load::<JobOpportunity>(conn)
            .await?;

        let comp = Self::find_company(conn, pcompany_id).await?;

        let result = jobs
            .into_iter()
            .map(|job| JobOpportunityWithCompany {
                id: job.id,
                title: job.title,
                description: job.description,
                category: job.category,
                address: job.address,
                latitude: job.latitude,
                longitude: job.longitude,
                start_date_time: job.start_date_time,
                duration_in_hours: job.duration_in_hours,
                pay_rate: job.pay_rate,
                status: job.status,
                company_id: job.company_id,
                company_name: Some(comp.name.clone()),
                company_logo_url: Some(comp.logo_url.clone()),
            })
            .collect();

        Ok(result)
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

    pub async fn find_employe(
        conn: &mut AsyncPgConnection,
        employe_id: &i64,
    ) -> Result<Employee, diesel::result::Error> {
        use crate::infrastructure::schema::employees::dsl::*;
        let employee = employees
            .filter(id.eq(employe_id))
            .first::<Employee>(conn)
            .await?;

        Ok(employee)
    }

    pub async fn find_company(
        conn: &mut AsyncPgConnection,
        company_id: &i64,
    ) -> Result<Company, diesel::result::Error> {
        use crate::infrastructure::schema::companies::dsl::*;
        let company = companies
            .filter(id.eq(company_id))
            .first::<Company>(conn)
            .await?;

        Ok(company)
    }
}
