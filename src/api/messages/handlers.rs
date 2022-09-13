use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::types::{Message, Metadata};
use crate::types::{User, UserFromDB};
use crate::{extractors::Claims, DB};
use actix_web::HttpRequest;
use actix_web::{get, web, Responder};
use sqlx::{query, query_as};

const VERSION: &str = "0.1.0";
const BRANCH: &str = "main";

macro_rules! fetch_db {
    ($db:expr) => {
        ($db.get()).await.try_acquire().unwrap()
    };
}

macro_rules! response {
    ($message:expr) => {
        web::Json(Message {
            metadata: Metadata {
                api: VERSION.to_string(),
                branch: BRANCH.to_string(),
            },
            text: $message.to_string(),
        })
    };
}

#[get("/admin")]
pub async fn admin(claims: Claims) -> impl Responder {
    response!(format!("admin message {}", claims.sub))
}

#[get("/protected")]
pub async fn protected(claims: Claims) -> impl Responder {
    response!(format!("This is a protected message. {}", claims.sub))
}

#[get("/public")]
pub async fn public() -> impl Responder {
    response!(format!("public"))
}

#[get("/ping")]
pub async fn ping() -> impl Responder {
    response!(format!("{:?}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis()
    ))
}

#[get("/taken")]
pub(crate) async fn user_taken(req: HttpRequest) -> impl Responder {
    if let Some(v) = req.headers().get("NewUsername") {
        let decoded = match base64::decode(v) {
            Ok(v) => v,
            Err(_) => return response!("invalid base64"),
        };
        let result = query!("select * from users where username == $1", decoded)
            .fetch_optional(&mut fetch_db!(DB))
            .await;
        if let Ok(Some(v)) = result {
            return response!(format!("{:?}", v))
        }
    }
    response!("not taken")
}

#[get("/self")]
pub async fn user_self(claims: Claims) -> impl Responder {
    let result = query_as!(UserFromDB, "select * from users where id == $1", claims.sub)
        .fetch_optional(&mut fetch_db!(DB))
        .await;
    if let Ok(Some(v)) = result {
        let formated: User = v.into();
        return response!(serde_json::to_string(&formated).unwrap());
    }
    response!("fail")
}

#[get("/new")]
pub async fn user_new(claims: Claims, req: HttpRequest) -> impl Responder {
    println!("claims {:?}", claims);
    println!("req {:?}", req);
    web::Json(Message {
        metadata: Metadata {
            api: VERSION.to_owned(),
            branch: BRANCH.to_owned(),
        },
        text: format!("This is an admin message. {}", claims.sub),
    })
}
