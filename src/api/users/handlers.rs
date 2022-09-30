use crate::api::types::{Message, Metadata};
use crate::extractors::Claims;
use crate::types::{User, UserFromDB};
use crate::{fetch_db, response};
use crate::{time, Database, BRANCH, VERSION};
use actix_web::{
    get,
    web::{self, Data},
    Responder,
};
use actix_web::{HttpRequest, HttpResponse};
use sqlx::{query, query_as, pool::PoolConnection, Sqlite};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

pub(crate) async fn is_username_taken(b64_username: &[u8], db: &mut PoolConnection<Sqlite>) -> bool {
    let decoded = match base64::decode(b64_username) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let decoded = String::from_utf8_lossy(&decoded);
    let result = query!("select * from users where username == $1", decoded)
        .fetch_optional(db)
        .await;
    matches!(result, Ok(Some(_)))
}

#[get("/taken")]
pub(crate) async fn user_taken(req: HttpRequest) -> impl Responder {
    if let Some(v) = req.headers().get("data") {
        // let decoded = match base64::decode(v) {
        //     Ok(v) => v,
        //     Err(_) => return response!("invalid base64"),
        // };
        // let mut db = fetch_db!(req);
        // let decoded = String::from_utf8_lossy(&decoded);
        // let result = query!("select * from users where username == $1", decoded)
        //     .fetch_optional(&mut db)
        //     .await;
        // if let Ok(Some(v)) = result {
        //     return response!(format!("{:?}", v));
        // }
        let mut db = fetch_db!(req);
        return if is_username_taken(v.as_bytes(), &mut db).await {
            response!("taken")
        } else {
            response!("not taken")
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
        // decode string first
        let data: User = serde_json::from_str(v.to_str().unwrap()).unwrap();
        let mut db = fetch_db!(req);
        if is_username_taken(data.username.as_bytes(), &mut db).await {
            return response!("failed to create new user");
        }
        let data = User {
            id: claims.sub,
            last_played: VecDeque::new(),
            followers: vec![],
            following: vec![],
            lastupdate: time!(),
            ..data
        };
        let data: UserFromDB = data.into();
        let _ = query!("insert into users(id, username, serverside, thumbnails, autoplay, allow_followers, public_account, activity, last_played, display_name, followers, following, analytics, lastupdate) values($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)", data.id, data.username, data.serverside, data.thumbnails, data.autoplay, data.allow_followers, data.public_account, data.activity, data.last_played, data.display_name, data.followers, data.following, data.analytics, data.lastupdate).execute(&mut db).await;
    }
    response!("done")
}

#[get("/edit")]
pub async fn edit(claims: Claims, req: HttpRequest) -> impl Responder {
    if let Some(v) = req.headers().get("Data") {
        // decode string first
        let data: User = serde_json::from_str(v.to_str().unwrap()).unwrap();
        let mut db = fetch_db!(req);
        if is_username_taken(data.username.as_bytes(), &mut db).await {
            return response!("failed to create new user");
        }
        let data = User {
            id: claims.sub,
            last_played: VecDeque::new(),
            followers: vec![],
            following: vec![],
            lastupdate: time!(),
            ..data
        };
        let data: UserFromDB = data.into();
        let _ = query!("insert into users(id, username, serverside, thumbnails, autoplay, allow_followers, public_account, activity, last_played, display_name, followers, following, analytics, lastupdate) values($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)", data.id, data.username, data.serverside, data.thumbnails, data.autoplay, data.allow_followers, data.public_account, data.activity, data.last_played, data.display_name, data.followers, data.following, data.analytics, data.lastupdate).execute(&mut db).await;
    }
    response!("done")
}

#[get("/follow/{user}")]
pub async fn follow(claims: Claims, user: web::Path<String>, req: HttpRequest) -> impl Responder {
    let mut db = fetch_db!(req);
    let result = query_as!(UserFromDB, "select * from users where id == $1", claims.sub)
        .fetch_optional(&mut db)
        .await;
    if let Ok(Some(v)) = result {
        let updated = UserFromDB::follow(&v.followers, &user);
        return if query!(
            "update users set followers = $1 where id = $2",
            updated,
            claims.sub
        )
        .fetch_optional(&mut db)
        .await
        .is_ok()
        {
            HttpResponse::Ok()
        } else {
            HttpResponse::BadRequest()
        };
    }
    HttpResponse::BadRequest()
}

#[get("/unfollow/{user}")]
pub async fn unfollow(claims: Claims, user: web::Path<String>, req: HttpRequest) -> impl Responder {
    let mut db = fetch_db!(req);
    let result = query_as!(UserFromDB, "select * from users where id == $1", claims.sub)
        .fetch_optional(&mut db)
        .await;
    if let Ok(Some(v)) = result {
        let updated = UserFromDB::unfollow(&v.followers, &user);
        return if query!(
            "update users set followers = $1 where id = $2",
            updated,
            claims.sub
        )
        .fetch_optional(&mut db)
        .await
        .is_ok()
        {
            HttpResponse::Ok()
        } else {
            HttpResponse::BadRequest()
        };
    }
    HttpResponse::BadRequest()
}

// TODO UPDATE SCHEMA
#[get("/toptracks")]
pub async fn toptracks(claims: Claims, req: HttpRequest) -> impl Responder {
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
        return if query!(
            "update users set last_played = $1 where id = $2",
            updated,
            claims.sub
        )
        .fetch_optional(&mut db)
        .await
        .is_ok()
        {
            HttpResponse::Ok()
        } else {
            HttpResponse::BadRequest()
        };
    }
    HttpResponse::BadRequest()
}
