use actix_web::body::MessageBody;
use actix_web::dev::Transform;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse};

use actix_web::http::StatusCode;

use actix_web::{Error, HttpResponse, ResponseError};
use std::fmt;

use futures_util::future::LocalBoxFuture;
use jwt_compact::alg::{Hs256, Hs256Key};
use jwt_compact::{AlgorithmExt, UntrustedToken, ValidationError};
use laguna_backend_dto::user::UserDTO;
use laguna_backend_model::role::Role;
use std::future::ready;
use std::future::Ready;

use crate::consts::ACCESS_TOKEN_HEADER_NAME;

pub struct AuthorizationMiddlewareFactory {
    min_role: Role,
    key: Hs256Key,
}

impl AuthorizationMiddlewareFactory {
    pub fn new(key: Hs256Key, min_role: Role) -> Self {
        Self { key, min_role }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthorizationMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static + MessageBody,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthorizationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthorizationMiddleware {
            min_role: self.min_role.clone(),
            key: self.key.clone(),
            service,
        }))
    }
}

pub struct AuthorizationMiddleware<S> {
    min_role: Role,
    key: Hs256Key,
    service: S,
}

#[derive(Debug)]
pub enum AuthorizationError {
    UnauthorizedRole { min_role: Role, actual_role: Role },
    InvalidToken(ValidationError),
}

impl From<ValidationError> for AuthorizationError {
    fn from(value: ValidationError) -> Self {
        Self::InvalidToken(value)
    }
}

impl fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnauthorizedRole {
                min_role,
                actual_role,
            } => {
                write!(
                    f,
                    "Unauthorized role. Expected: {:?}, actual: {:?}",
                    min_role, actual_role
                )
            }
            Self::InvalidToken(validation_error) => {
                write!(f, "{}", validation_error)
            }
        }
    }
}

impl ResponseError for AuthorizationError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidToken(_) => StatusCode::BAD_REQUEST,
            Self::UnauthorizedRole { .. } => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Self::InvalidToken(_) => HttpResponse::BadRequest().body(format!("{}", self)),
            Self::UnauthorizedRole { .. } => HttpResponse::Unauthorized().body(format!("{}", self)),
        }
    }
}

impl<S, B> Service<ServiceRequest> for AuthorizationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static + MessageBody,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Token has already been validated & verified by AuthenticationService by the time it reaches this middleware.
        let access_token_header = req.headers().get(ACCESS_TOKEN_HEADER_NAME);
        if let Some(access_token_header) = access_token_header {
            // SECURITY: Token is trusted at this point but additional verification is however still performed.
            // NOTE: This is probably not a huge bottleneck and is a consequence of using external libraries for authentication (not authorization).
            let access_token = UntrustedToken::new(access_token_header.to_str().unwrap()).unwrap();
            let integrity = Hs256.validate_integrity::<UserDTO>(&access_token, &self.key);
            if let Err(validation_error) = integrity {
                return Box::pin(async move {
                    Result::<Self::Response, Self::Error>::Err(
                        AuthorizationError::from(validation_error).into(),
                    )
                });
            }
            // SAFETY: .unwrap() is safe because we checked for error.
            let integrity = integrity.unwrap();
            let min_role = self.min_role;
            let role = integrity.claims().custom.role;
            if role < min_role {
                return Box::pin(async move {
                    Result::<Self::Response, Self::Error>::Err(
                        AuthorizationError::UnauthorizedRole {
                            min_role,
                            actual_role: role,
                        }
                        .into(),
                    )
                });
            }
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}