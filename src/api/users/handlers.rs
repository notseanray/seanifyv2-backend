use crate::api::types::{Message, Metadata};
use crate::api::BoolResult;
use crate::extractors::Claims;
use crate::types::{User, UserFromDB};
use crate::DB;
use crate::{fetch_db, response};
use crate::{time, BRANCH, VERSION};
use actix_web::{get, web, Responder};
use actix_web::{HttpRequest, HttpResponse};
use reqwest::StatusCode;
use sqlx::{pool::PoolConnection, query, query_as, Sqlite};
use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) async fn is_username_taken(username: &str, db: &mut PoolConnection<Sqlite>) -> bool {
    let result = query!("select * from users where username == $1", username)
        .fetch_optional(db)
        .await;
    matches!(result, Ok(Some(_)))
}

#[get("/taken")]
pub(crate) async fn user_taken(req: HttpRequest) -> impl Responder {
    if let Some(v) = req.headers().get("data") {
        let mut db = fetch_db!();
        if is_username_taken(v.to_str().unwrap_or_default(), &mut db).await {
            HttpResponse::NotAcceptable()
        } else {
            HttpResponse::Ok()
        }
    } else {
        HttpResponse::BadRequest()
    }
}

#[get("/self")]
pub async fn user_self(claims: Claims) -> impl Responder {
    let mut db = fetch_db!();
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
    if let Some(v) = req.headers().get("data") {
        // TODO ERROR HANDLING
        let data: User = serde_json::from_str(v.to_str().unwrap()).unwrap();
        let mut db = fetch_db!();
        if is_username_taken(&data.username, &mut db).await {
            return response!("failed to create new user");
        }
        let data = User {
            id: claims.sub,
            last_played: VecDeque::new(),
            followers: vec![],
            following: vec![],
            lastupdate: time!(),
            admin: false,
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
        let mut db = fetch_db!();
        if is_username_taken(&data.username, &mut db).await {
            return response!("failed to create new user");
        }
        let result = query_as!(UserFromDB, "select * from users where id == $1", claims.sub)
            .fetch_optional(&mut db)
            .await;
        if let Ok(Some(v)) = result {
            let formated: User = v.into();
            let data = User {
                id: claims.sub,
                last_played: VecDeque::new(),
                followers: formated.followers,
                following: formated.following,
                lastupdate: time!(),
                admin: formated.admin,
                ..data
            };
            let data: UserFromDB = data.into();
            let _ = query!("insert into users(id, username, serverside, thumbnails, autoplay, allow_followers, public_account, activity, last_played, display_name, followers, following, analytics, lastupdate) values($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)", data.id, data.username, data.serverside, data.thumbnails, data.autoplay, data.allow_followers, data.public_account, data.activity, data.last_played, data.display_name, data.followers, data.following, data.analytics, data.lastupdate).execute(&mut db).await;
        }
    }
    response!("done")
}

#[get("/follow/{user}")]
pub async fn follow(claims: Claims, user: web::Path<String>) -> impl Responder {
    let mut db = fetch_db!();
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
pub async fn unfollow(claims: Claims, user: web::Path<String>) -> impl Responder {
    let mut db = fetch_db!();
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

#[get("/delete")]
pub async fn delete(claims: Claims) -> impl Responder {
    let mut db = fetch_db!();
    if query!("delete from users where id = $1", claims.sub)
        .execute(&mut db)
        .await
        .is_ok()
    {
        HttpResponse::new(StatusCode::OK)
    } else {
        HttpResponse::new(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[get("/delete/{username}")]
pub async fn delete_user(claims: Claims, username: web::Path<String>) -> impl Responder {
    let username = username.to_string();
    if username.contains('`') {
        return HttpResponse::BadRequest();
    }
    let mut db = fetch_db!();
    if let Ok(Some(v)) = query_as!(
        BoolResult,
        "select admin from users where id == $1 and admin == true",
        claims.sub
    )
    .fetch_optional(&mut db)
    .await
    {
        if v.admin {
            return if query!("delete from users where username = $1", username)
                .execute(&mut db)
                .await
                .is_ok()
            {
                HttpResponse::Ok()
            } else {
                HttpResponse::BadRequest()
            };
        }
    }
    HttpResponse::BadRequest()
}

#[get("/listen/{song}")]
pub async fn listen(claims: Claims, song: web::Path<String>) -> impl Responder {
    if song.contains('`') {
        return HttpResponse::BadRequest();
    }
    let mut db = fetch_db!();
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

#[get("/get/id/{user}")]
pub async fn get_user_from_id(claims: Claims, user: web::Path<String>) -> impl Responder {
    let user = user.to_string();
    let mut db = fetch_db!();
    if let Some(v) = UserFromDB::from_id(&mut db, &user).await {
        let v: User = v.into();
        if !v.public_account {
            if user == claims.sub {
                return serde_json::to_string(&v).unwrap();
            } else {
                return "".to_string();
            }
        }
        return serde_json::to_string(&v).unwrap();
    }
    "".to_string()
}

#[get("/get/name/{user}")]
pub async fn get_user_from_name(claims: Claims, user: web::Path<String>) -> impl Responder {
    let user = user.to_string();
    let mut db = fetch_db!();
    if let Some(v) = UserFromDB::from_username(&mut db, &user).await {
        let v: User = v.into();
        if !v.public_account {
            if v.id == claims.sub {
                return serde_json::to_string(&v).unwrap();
            } else {
                return "".to_string();
            }
        }
        return serde_json::to_string(&v).unwrap();
    }
    "".to_string()
}
