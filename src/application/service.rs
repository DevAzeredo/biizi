use axum::Json;
use chrono::Utc;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use crate::{
    domain::models::{
        Company, Employee, JobOpportunity, NewCompany, NewEmployee, NewJobOpportunity, NewUser,
        User,
    },
    infrastructure::{auth::Claims, repositories::Repository},
};

pub struct Service;
const SECRET: &str = "senhafortexD";
impl Service {
    pub async fn validate_jwt_and_get_user(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        token: &str,
    ) -> Result<User, String> {
        let claims = match Self::validate_jwt(token) {
            Ok(claims) => claims,
            Err(err) => return Err(format!("Invalid token: {}", err)),
        };

        let user = match Repository::find_by_login(conn, &claims.sub).await {
            Ok(user) => user,
            Err(_) => return Err("User not found".to_string()),
        };

        Ok(user)
    }
    pub async fn find_by_login(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        user_login: &str,
    ) -> Result<User, diesel::result::Error> {
        Repository::find_by_login(conn, user_login).await
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
        let new_job = NewJobOpportunity {
            company_id: user.companyid.unwrap_or_else(|| 0),
            ..job.clone()
        };
        Repository::save_job_opportunity(conn, &new_job).await
    }

    pub fn generate_jwt(user: &User) -> Result<String, jsonwebtoken::errors::Error> {
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .unwrap()
            .timestamp() as usize;

        let claims = Claims {
            sub: user.login.clone(),
            exp: expiration,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(SECRET.as_ref()),
        )
    }

    pub async fn register_user(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        new_user: NewUser,
    ) -> Result<String, diesel::result::Error> {
        match Repository::find_by_login(conn, &new_user.login).await {
            Ok(_) => return Err(diesel::result::Error::BrokenTransactionManager),
            Err(_) => {}
        }
        let user = Repository::save_user(conn, &new_user).await?;
        let token = match Self::generate_jwt(&user) {
            Ok(token) => Ok(token),
            Err(_) => Err(diesel::result::Error::BrokenTransactionManager),
        };
        Ok(token?)
    }

    pub fn validate_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims)
    }

    pub async fn authenticate(
        conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        login: &str,
        password: &str,
    ) -> Result<User, diesel::result::Error> {
        let user = Repository::find_by_login(conn, login).await?;
        match Self::verify_password(password, &user.password) {
            Ok(true) => Ok(user),
            _ => Err(diesel::result::Error::NotFound),
        }
    }

    pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
        bcrypt::verify(password, hash)
    }
}
