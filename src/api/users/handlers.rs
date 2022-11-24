use crate::api::types::{Message, Metadata};
use crate::api::BoolResult;
use crate::extractors::Claims;
use crate::types::User;
use crate::DB;
use crate::{fetch_db, response};
use crate::{time, BRANCH, VERSION};
use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, Responder};
use actix_web::{HttpRequest, HttpResponse};
use futures::TryStreamExt;
use sqlx::{pool::PoolConnection, query, query_as, Postgres};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[post("/upload_pfp")]
pub(crate) async fn upload_pfp(
    req: HttpRequest,
    claims: Claims,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    // once we have the first check we don't have to keep getting the filename
    if claims.sub.contains('/') {
        return Ok(HttpResponse::BadRequest().into());
    }
    let Some(mime_type) = req.headers().get("ContentType") else {
        return Ok(HttpResponse::BadRequest().into());
    };
    let file_extension = match mime_type.to_str() {
        Ok("image/png" | "png") => "png",
        Ok("image/jpeg" | "jpeg") => "jpg",
        _ => return Ok(HttpResponse::BadRequest().into()),
    };
    if !Path::new("./profiles").exists() && fs::create_dir_all("./profiles").is_err() {
        return Ok(HttpResponse::InternalServerError().into());
    }
    let path = Arc::new(format!("./profiles/{}.{}", &claims.sub, file_extension));
    let mut init_part = false;
    let mut filename = String::new();
    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();
        let Some(form_file_name) = content_disposition.get_filename() else {
            return Ok(HttpResponse::BadRequest().into());
        };
        if !init_part {
            filename = form_file_name.to_string();
            init_part = true;
        } else if form_file_name != filename.as_str() {
            // if all the chunks don't have the same file name we have an issue
            return Ok(HttpResponse::BadRequest().into());
        }

        let path = path.clone();
        // blocking op, use threadpool
        let mut f = web::block(move || std::fs::File::create(&*path)).await??;

        while let Some(chunk) = field.try_next().await? {
            // blocking op, again using threadpool
            f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
        }
    }
    Ok(HttpResponse::Ok().into())
}

#[post("/delete_pfp")]
pub async fn delete_pfp(claims: Claims) -> impl Responder {
    let _ = web::block(move || {
        let path = format!("./profiles/{}/.", claims.sub);
        if Path::new(&(path.to_owned() + "png")).exists() {
            let _ = fs::remove_file(&path);
        }
        if Path::new(&(path.to_owned() + "jpg")).exists() {
            let _ = fs::remove_file(&path);
        }
    })
    .await;
    HttpResponse::Ok()
}

pub(crate) async fn is_username_taken(username: &str, db: &mut PoolConnection<Postgres>) -> bool {
    let result = query!("select * from users where username = $1", username)
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
    let result = query_as!(User, "select * from users where id = $1", claims.sub)
        .fetch_optional(&mut db)
        .await;
    if let Ok(Some(v)) = result {
        return response!(serde_json::to_string(&v).unwrap());
    }
    response!("")
}

#[get("/new")]
pub async fn user_new(claims: Claims, req: HttpRequest) -> impl Responder {
    if let Some(v) = req.headers().get("data") {
        // TODO ERROR HANDLING
        let data: User = serde_json::from_str(v.to_str().unwrap()).unwrap();
        let mut db = fetch_db!();
        if is_username_taken(&data.username, &mut db).await {
            return HttpResponse::NotAcceptable();
        }
        let data = User {
            id: claims.sub,
            last_played: vec![],
            followers: vec![],
            following: vec![],
            lastupdate: time!(),
            admin: false,
            ..data
        };
        let _ = query!(
            r#"insert into users(
                id,
                username,
                serverside,
                thumbnails,
                autoplay,
                allow_followers,
                public_account,
                activity,
                last_played,
                display_name,
                followers,
                following,
                analytics,
                lastupdate)
            values($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"#,
            data.id,
            data.username,
            data.serverside,
            data.thumbnails,
            data.autoplay,
            data.allow_followers,
            data.public_account,
            data.activity,
            &data.last_played,
            data.display_name,
            &data.followers,
            &data.following,
            data.analytics,
            data.lastupdate
        )
        .execute(&mut db)
        .await;
    }
    HttpResponse::Ok()
}

#[get("/edit")]
pub async fn edit(claims: Claims, req: HttpRequest) -> impl Responder {
    if let Some(v) = req.headers().get("Data") {
        // decode string first
        let data: User = serde_json::from_str(v.to_str().unwrap()).unwrap();
        let mut db = fetch_db!();
        if is_username_taken(&data.username, &mut db).await {
            return HttpResponse::NotAcceptable();
        }
        let result = query_as!(User, "select * from users where id = $1", claims.sub)
            .fetch_optional(&mut db)
            .await;
        if let Ok(Some(v)) = result {
            let data = User {
                id: claims.sub,
                last_played: vec![],
                followers: v.followers,
                following: v.following,
                lastupdate: time!(),
                admin: v.admin,
                ..data
            };
            let _ = query!(
                "insert into users(
                    id, 
                    username, 
                    serverside, 
                    thumbnails, 
                    autoplay, 
                    allow_followers, 
                    public_account, 
                    activity, 
                    last_played, 
                    display_name, 
                    followers, 
                    following, 
                    analytics, 
                    lastupdate) 
                values($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
                data.id,
                data.username,
                data.serverside,
                data.thumbnails,
                data.autoplay,
                data.allow_followers,
                data.public_account,
                data.activity,
                &data.last_played,
                data.display_name,
                &data.followers,
                &data.following,
                data.analytics,
                data.lastupdate
            )
            .execute(&mut db)
            .await;
        }
    }
    HttpResponse::Ok()
}

#[get("/follow/{user}")]
pub async fn follow(claims: Claims, user: web::Path<String>) -> impl Responder {
    let mut db = fetch_db!();
    let result = query_as!(User, "select * from users where id = $1", claims.sub)
        .fetch_optional(&mut db)
        .await;
    if let Ok(Some(mut v)) = result {
        v.follow(user.to_string());
        return if query!(
            "update users set followers = $1 where id = $2",
            &v.followers,
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
    let result = query_as!(User, "select * from users where id = $1", claims.sub)
        .fetch_optional(&mut db)
        .await;
    if let Ok(Some(mut v)) = result {
        v.unfollow(&user);
        return if query!(
            "update users set followers = $1 where id = $2",
            &v.followers,
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
        HttpResponse::Ok()
    } else {
        HttpResponse::ServiceUnavailable()
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
        "select admin from users where id = $1 and admin = true",
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
    let result = query_as!(User, "select * from users where id = $1", claims.sub)
        .fetch_optional(&mut db)
        .await;
    if let Ok(Some(mut v)) = result {
        v.now_playing(song.to_string());
        return if query!(
            "update users set last_played = $1 where id = $2",
            &v.last_played,
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
    if let Some(v) = User::from_id(&mut db, &user).await {
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
    if let Some(v) = User::from_username(&mut db, &user).await {
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
