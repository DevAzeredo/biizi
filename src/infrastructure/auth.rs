use axum::{
    body::Body,
    extract::{Json, Request, State},
    http::{self, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{application::service::Service, Pool};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub login: String,
}

pub struct AuthError {
    message: String,
    status_code: StatusCode,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response<Body> {
        let body = Json(json!({ "error": self.message }));
        (self.status_code, body).into_response()
    }
}

pub struct Auth;

impl Auth {
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
        verify(password, hash)
    }

    pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
        hash(password, DEFAULT_COST)
    }

    pub fn encode_jwt(login: String) -> Result<String, StatusCode> {
        let secret = "randomstring".to_string();
        let now = Utc::now();
        let expire = Duration::hours(24);
        let exp = (now + expire).timestamp() as usize;
        let iat = now.timestamp() as usize;

        let claim = Claims { iat, exp, login };

        encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn decode_jwt(jwt: String) -> Result<TokenData<Claims>, StatusCode> {
        let secret = "randomstring".to_string();

        decode(
            &jwt,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn authorize(
        State(pool): State<Pool>,
        mut req: Request<Body>,
        next: Next,
    ) -> Result<Response<Body>, AuthError> {
        let auth_header = req.headers_mut().get(http::header::AUTHORIZATION);

        let auth_header = match auth_header {
            Some(header) => header.to_str().map_err(|_| AuthError {
                message: "Empty header is not allowed".to_string(),
                status_code: StatusCode::FORBIDDEN,
            })?,
            None => {
                return Err(AuthError {
                    message: "Please add the JWT token to the header".to_string(),
                    status_code: StatusCode::FORBIDDEN,
                })
            }
        };

        let mut header = auth_header.split_whitespace();
        let (bearer, token) = (header.next(), header.next());

        let token_data = match Auth::decode_jwt(token.unwrap().to_string()) {
            Ok(data) => data,
            Err(_) => {
                return Err(AuthError {
                    message: "Unable to decode token".to_string(),
                    status_code: StatusCode::UNAUTHORIZED,
                })
            }
        };

        let mut conn = pool.get().await.map_err(|_| AuthError {
            message: "Unable to connect to database".to_string(),
            status_code: StatusCode::FORBIDDEN,
        })?;
        let current_user = Service::find_by_login(&mut conn, &token_data.claims.login)
            .await
            .map_err(|_| AuthError {
                message: "Unable to find user".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            })?;

        req.extensions_mut().insert(current_user);
        Ok(next.run(req).await)
    }
}

#[derive(Deserialize)]
pub struct SignInData {
    pub login: String,
    pub password: String,
}

pub async fn sign_in(
    conn: &mut bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
    user_data: SignInData,
) -> Result<String, StatusCode> {
    let user = Service::find_by_login(conn, &user_data.login)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !Auth::verify_password(&user_data.password, &user.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = Auth::encode_jwt(user.login).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(token)
}
