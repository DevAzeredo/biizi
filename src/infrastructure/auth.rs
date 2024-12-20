use axum::{
    extract::{Request, State},
    http::{self, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};

use crate::{Pool, Service};
use serde::{Deserialize, Serialize};
use serde_json::json;
pub struct AuthError {
    message: String,
    status_code: StatusCode,
}
impl IntoResponse for AuthError {
    fn into_response(self) -> Response<Body> {
        let body = Json(json!({
            "error": self.message,
        }));

        (self.status_code, body).into_response()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub sub: String,
}
pub async fn authorize(
    State(pool): State<Pool>,
    mut req: Request,
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

    let (_, token) = (header.next(), header.next());

    let token_data = match Service::validate_jwt(token.unwrap()) {
        Ok(data) => data,
        Err(_) => {
            return Err(AuthError {
                message: "Unable to decode token".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            })
        }
    };
    let mut conn = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(AuthError {
                message: e.to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    };
    let user = match Service::find_by_login(&mut conn, &token_data.sub).await {
        Ok(user) => user,
        Err(e) => {
            return Err(AuthError {
                message: e.to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    };
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}
