use crate::types::ErrorMessage;
use actix_web::{
    error::ResponseError,
    http::{StatusCode, Uri},
    Error, FromRequest, HttpResponse,
};
use actix_web_httpauth::{
    extractors::bearer::BearerAuth, headers::www_authenticate::bearer::Bearer,
};
use derive_more::Display;
use jsonwebtoken::{
    decode, decode_header,
    jwk::{self, AlgorithmParameters, JwkSet},
    Algorithm, DecodingKey, Validation,
};
use serde::Deserialize;
use std::str;
use std::{collections::HashSet, future::Future, pin::Pin};

#[derive(Clone, Deserialize)]
pub struct Auth0Config {
    audience: String,
    domain: String,
}

impl Default for Auth0Config {
    fn default() -> Self {
        envy::prefixed("AUTH0_")
            .from_env()
            .expect("Provide missing environment variables for Auth0Client")
    }
}

#[derive(Debug, Display)]
enum ClientError {
    #[display(fmt = "authentication")]
    Authentication(actix_web_httpauth::extractors::AuthenticationError<Bearer>),
    #[display(fmt = "decode")]
    Decode(jsonwebtoken::errors::Error),
    #[display(fmt = "not_found")]
    NotFound(String),
    #[display(fmt = "unsupported_algorithm")]
    UnsupportedAlgortithm(AlgorithmParameters),
    #[display(fmt = "invalid user id")]
    InvalidUserID(String),
    #[display(fmt = "invalid json")]
    InvalidJson,
    #[display(fmt = "invalid issuer url")]
    InvalidIssuerUrl,
}

impl ResponseError for ClientError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::Authentication(_) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: None,
                error_description: None,
                message: "Requires authentication".to_string(),
            }),
            Self::Decode(_) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("invalid_token".to_string()),
                error_description: Some(
                    "Authorization header value must follow this format: Bearer access-token"
                        .to_string(),
                ),
                message: "Bad credentials".to_string(),
            }),
            Self::NotFound(msg) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("invalid_token".to_string()),
                error_description: Some(msg.to_string()),
                message: "Bad credentials".to_string(),
            }),
            Self::UnsupportedAlgortithm(alg) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("invalid_token".to_string()),
                error_description: Some(format!(
                    "Unsupported encryption algortithm expected RSA got {:?}",
                    alg
                )),
                message: "Bad credentials".to_string(),
            }),
            Self::InvalidUserID(msg) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("invalid_user_id".to_string()),
                error_description: Some(msg.to_string()),
                message: "invalid user id".to_string(),
            }),
            Self::InvalidJson => HttpResponse::BadRequest().json(ErrorMessage {
                error: Some("invalid_json_recieved_for_jwt".to_string()),
                error_description: None,
                message: "invalid json recieved for jwt".to_string(),
            }),
            Self::InvalidIssuerUrl => HttpResponse::BadRequest().json(ErrorMessage {
                error: Some("invalid_issuer_url".to_string()),
                error_description: None,
                message: "failed to construct issuer url".to_string(),
            }),
        }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

#[derive(Debug, Deserialize)]
pub struct Claims {
    _permissions: Option<HashSet<String>>,
    pub sub: String,
}

impl FromRequest for Claims {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let config = req.app_data::<Auth0Config>().unwrap().clone();
        let extractor = BearerAuth::extract(req);
        Box::pin(async move {
            let credentials = extractor.await.map_err(ClientError::Authentication)?;
            let token = credentials.token();
            let header = decode_header(token).map_err(ClientError::Decode)?;
            let kid = header.kid.ok_or_else(|| {
                ClientError::NotFound("kid not found in token header".to_string())
            })?;
            let domain = config.domain.as_str();
            let Ok(jwks) = reqwest::get(format!("https://{domain}/.well-known/jwks.json")).await else {
                Err(ClientError::NotFound("kid not found in token header".to_string()))?
            };
            let Ok(jwks) = jwks.text().await else {
                Err(ClientError::NotFound("kid not found in token header".to_string()))?
            };
            let Ok(jwks): Result<JwkSet, _> = serde_json::from_str(&jwks) else {
                Err(ClientError::InvalidJson)?
            };
            let jwk = jwks
                .find(&kid)
                .ok_or_else(|| ClientError::NotFound("No JWK found for kid".to_string()))?;
            match jwk.clone().algorithm {
                AlgorithmParameters::RSA(ref rsa) => {
                    let mut validation = Validation::new(Algorithm::RS256);
                    validation.set_audience(&[config.audience]);
                    let Ok(issuer) = Uri::builder()
                        .scheme("https")
                        .authority(domain)
                        .path_and_query("/")
                        .build() else {
                            return Err(ClientError::InvalidIssuerUrl)?
                        };
                    validation.set_issuer(&[issuer]);
                    let key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                        .map_err(ClientError::Decode)?;
                    let token =
                        decode::<Claims>(token, &key, &validation).map_err(ClientError::Decode)?;
                    Ok(token.claims)
                }
                algorithm => Err(ClientError::UnsupportedAlgortithm(algorithm).into()),
            }
        })
    }
}
