use crate::api::types::{Metadata, Message};
use crate::extractors::Claims;
use crate::types::{User, UserFromDB};
use crate::{Database, BRANCH, VERSION};
use actix_web::{HttpRequest, HttpResponse};
use actix_web::{
    get,
    web::{self, Data},
    Responder,
};
use sqlx::{query, query_as};
use crate::{fetch_db, response};
use std::sync::Mutex;

// #[get("/admin")]
// pub async fn admin(claims: Claims) -> impl Responder {
//     response!(format!("admin message {}", claims.sub))
// }

#[get("/protected")]
pub async fn protected(claims: Claims) -> impl Responder {
    response!(format!("This is a protected message. {}", claims.sub))
}

#[get("/public")]
pub async fn public() -> impl Responder {
    response!(format!("public"))
}


// pub(crate) async check_taken()

#[get("/taken")]
pub(crate) async fn user_taken(req: HttpRequest) -> impl Responder {
    if let Some(v) = req.headers().get("NewUsername") {
        let decoded = match base64::decode(v) {
            Ok(v) => v,
            Err(_) => return response!("invalid base64"),
        };
        let mut db = fetch_db!(req);
        let decoded = String::from_utf8_lossy(&decoded);
        let result = query!("select * from users where username == $1", decoded)
            .fetch_optional(&mut db)
            .await;
        if let Ok(Some(v)) = result {
            return response!(format!("{:?}", v));
        }
    }
    response!("not taken")
}

#[get("/self")]
pub async fn user_self(req: HttpRequest, claims: Claims) -> impl Responder {
    let mut db = fetch_db!(req);
    let result = query_as!(UserFromDB, "select * from users where id == $1", claims.sub)
        .fetch_optional(&mut db)
        .await;
    if let Ok(Some(v)) = result {
        let formated: User = v.into();
        return response!(serde_json::to_string(&formated).unwrap());
    }
    response!("failed to fetch user data, does the account exist?")
}

#[get("/new")]
pub async fn user_new(claims: Claims, req: HttpRequest) -> impl Responder {
    if let Some(v) = req.headers().get("Data") {
        let data: User = serde_json::from_str(v.to_str().unwrap()).unwrap();

    }
    web::Json(Message {
        metadata: Metadata {
            api: VERSION.to_owned(),
            branch: BRANCH.to_owned(),
        },
        text: format!("This is an admin message. {}", claims.sub),
    })
}

#[get("/edit")]
pub async fn edit(claims: Claims, req: HttpRequest) -> impl Responder {
    "test".to_string()
}

#[get("/follow")]
pub async fn follow(claims: Claims, req: HttpRequest) -> impl Responder {
    "test".to_string()
}

#[get("/unfollow")]
pub async fn unfollow(claims: Claims, req: HttpRequest) -> impl Responder {
    "test".to_string()
}

#[get("/toptracks")]
pub async fn toptracks(claims: Claims, req: HttpRequest) -> impl Responder {
    "test".to_string()
}

#[get("/profile")]
pub async fn profile(claims: Claims, req: HttpRequest) -> impl Responder {
    "test".to_string()
}

#[get("/listen/{song}")]
pub async fn listen(claims: Claims, song: web::Path<String>, req: HttpRequest) -> impl Responder {
    let mut db = fetch_db!(req);
    let result = query_as!(UserFromDB, "select * from users where id == $1", claims.sub)
        .fetch_optional(&mut db)
        .await;
    if let Ok(Some(v)) = result {
        let updated = UserFromDB::now_playing(&v.last_played, &song);
        return if query!("update users set last_played = $1 where id = $2", updated, claims.sub).fetch_optional(&mut db).await.is_ok() {
            HttpResponse::Ok()
        } else {
            HttpResponse::BadRequest()
        }
    }
    HttpResponse::BadRequest()
}
