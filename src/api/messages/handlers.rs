use std::time::{SystemTime, UNIX_EPOCH, Duration};

use super::types::{Message, Metadata};
use actix_web::HttpRequest;
use sqlx::query;
use crate::{extractors::Claims, DB};
use actix_web::{get, web, Responder};

const VERSION: &str = "0.1.0";
const BRANCH: &str = "main";

#[get("/admin")]
pub async fn admin(claims: Claims) -> impl Responder {
    web::Json(Message {
        metadata: Metadata {
            api: VERSION.to_owned(),
            branch: BRANCH.to_owned(),
        },
        text: format!("This is an admin message. {}", claims.sub),
    })
}

#[get("/protected")]
pub async fn protected(claims: Claims) -> impl Responder {
    web::Json(Message {
        metadata: Metadata {
            api: VERSION.to_owned(),
            branch: BRANCH.to_owned(),
        },
        text: format!("This is a protected message. {}", claims.sub),
    })
}

#[get("/public")]
pub async fn public() -> impl Responder {
    web::Json(Message {
        metadata: Metadata {
            api: VERSION.to_owned(),
            branch: BRANCH.to_owned(),
        },
        text: "This is a public message.".to_string(),
    })
}

#[get("/ping")]
pub async fn ping() -> impl Responder {
    web::Json(Message {
        metadata: Metadata {
            api: VERSION.to_owned(),
            branch: BRANCH.to_owned(),
        },
        text: format!("{:?}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_millis()),
    })
}

#[get("/user/taken")]
pub(crate) async fn user_taken(req: HttpRequest) -> impl Responder {
    if let Some(v) = req.headers().get("NewUsername") {
        let decoded = base64::decode(v).ok().unwrap();
        query!("select * from users where ID == $1", decoded).fetch_one(&mut *(DB.get().await).write().unwrap()).await.unwrap();
    }
    web::Json(Message {
        metadata: Metadata {
            api: "v0.1.0".to_string(),
            branch: "production".to_string(),
        },
        text: format!(""),
    })
}
