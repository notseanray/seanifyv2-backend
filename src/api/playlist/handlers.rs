use crate::extractors::Claims;
use crate::types::{Playlist, User};
use crate::DB;
use crate::{fetch_db, time};
use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, Responder};
use actix_web::{HttpRequest, HttpResponse};
use futures::TryStreamExt;
use std::fs;
use std::io::Write;
use std::path::Path as stdpath;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use web::Path;

use sqlx::{query, query_as};

#[post("/upload_cover")]
pub(crate) async fn upload_cover(
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
    if !stdpath::new("./playlist").exists() && fs::create_dir_all("./playlist").is_err() {
        return Ok(HttpResponse::InternalServerError().into());
    }
    let path = Arc::new(format!("./playlist/{}.{}", &claims.sub, file_extension));
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

#[post("/delete_cover")]
pub async fn delete_cover(claims: Claims) -> impl Responder {
    let _ = web::block(move || {
        let path = format!("./profiles/{}/.", claims.sub);
        if stdpath::new(&(path.to_owned() + "png")).exists() {
            let _ = fs::remove_file(&path);
        }
        if stdpath::new(&(path.to_owned() + "jpg")).exists() {
            let _ = fs::remove_file(&path);
        }
    })
    .await;
    HttpResponse::Ok()
}

#[get("/new")]
pub async fn playlist_new(claims: Claims, req: HttpRequest) -> impl Responder {
    let (Some(v), Some(u)) = (
        req.headers().get("data"),
        User::from_id(&mut fetch_db!(), &claims.sub).await,
    ) else {
        return HttpResponse::Forbidden();
    };
    let mut data: Playlist = serde_json::from_str(v.to_str().unwrap()).unwrap();
    data.author_id = claims.sub;
    // get username from id
    data.author = u.username;
    data.description.truncate(400);
    data.name.truncate(100);
    data.likes.clear();
    data.duration = 0;
    data.lastupdate = time!();
    data.cover.truncate(2000);
    // TODO INSERT
    HttpResponse::Ok()
}

#[get("/{username}")]
pub async fn playlist_user_data(username: Path<String>, claims: Claims) -> impl Responder {
    let username = username.to_string();
    let mut db = fetch_db!();
    let Some(u) = User::from_id(&mut db, &claims.sub).await else {
        return "[]".to_string();
    };
    let playlist = query_as!(
        Playlist,
        "select * from playlist where author = $1",
        username,
    )
    .fetch_all(&mut db)
    .await;
    let Ok(v) = playlist else {
        return "[]".to_string();
    };
    let playlist: Vec<Playlist> = v
        .into_iter()
        .filter_map(|x| {
            if x.author_id == claims.sub
                || x.edit_list.contains(&username)
                || x.public_playlist
                || u.admin
            {
                Some(x)
            } else {
                None
            }
        })
        .collect();
    serde_json::to_string(&playlist).unwrap_or_default()
}

#[get("/{username}/{playlist_name}/hash")]
pub async fn playlist_hash(
    username: Path<String>,
    playlist_name: Path<String>,
    claims: Claims,
) -> impl Responder {
    let mut db = fetch_db!();
    let username = username.to_string();
    let Some(u) = User::from_id(&mut db, &claims.sub).await else {
        return String::new();
    };
    let playlist_name = playlist_name.to_string();
    // if they are requesting themself
    let playlist = query_as!(
        Playlist,
        "select * from playlist where author = $1 and name = $2",
        username,
        playlist_name,
    )
    .fetch_all(&mut db)
    .await;
    let Ok(v) = playlist else {
        return String::new();
    };
    let v: Vec<Playlist> = v
        .into_iter()
        .filter_map(|x| {
            if x.author_id == claims.sub
                || x.edit_list.contains(&username)
                || x.public_playlist
                || u.admin
            {
                Some(x)
            } else {
                None
            }
        })
        .collect();
    let mut hasher = blake3::Hasher::new();
    for ele in v {
        hasher.update(ele.songs.join("").as_bytes());
    }
    hasher.finalize().to_string()
}

#[get("/{username}/{playlist_name}/data")]
pub async fn playlist_data(
    username: Path<String>,
    playlist_name: Path<String>,
    claims: Claims,
) -> impl Responder {
    let mut db = fetch_db!();
    let Some(_u) = User::from_id(&mut db, &claims.sub).await else {
        return String::new();
    };
    let playlist_name = playlist_name.to_string();
    let username = username.to_string();
    // if they are requesting themself
    let playlist = query_as!(
        Playlist,
        "select * from playlist where author = $1 and name = $2",
        username,
        playlist_name
    )
    .fetch_all(&mut db)
    .await;
    let Ok(v) = playlist else {
        return String::new();
    };
    let playlist: Vec<Playlist> = v
        .into_iter()
        .filter_map(|x| {
            if x.author_id == claims.sub || x.edit_list.contains(&username) || x.public_playlist {
                Some(x)
            } else {
                None
            }
        })
        .collect();
    serde_json::to_string(&playlist).unwrap_or_default()
}

#[get("/{username}/{playlist_name}/like")]
pub async fn playlist_like(
    username: Path<String>,
    playlist_name: Path<String>,
    claims: Claims,
) -> impl Responder {
    let playlist_name = playlist_name.to_string();
    let username = username.to_string();
    let mut db = fetch_db!();
    let Some(u) = User::from_id(&mut db, &claims.sub).await else {
        return HttpResponse::Forbidden();
    };
    // if they are requesting themself
    let playlist = query_as!(
        Playlist,
        "select * from playlist where author = $1 and name = $2",
        username,
        playlist_name,
    )
    .fetch_optional(&mut db)
    .await;
    let Ok(Some(mut v)) = playlist else {
        return HttpResponse::BadRequest();
    };
    v.like(claims.sub.to_string());
    if !(v.author_id == claims.sub
        || v.edit_list.contains(&claims.sub)
        || v.public_playlist
        || u.admin)
    {
        return HttpResponse::Forbidden();
    }
    if query!(
        "update playlist set likes = $1 where author = $2 and name = $3",
        &v.likes,
        username,
        playlist_name
    )
    .execute(&mut db)
    .await
    .is_ok()
    {
        HttpResponse::Ok()
    } else {
        HttpResponse::Forbidden()
    }
}

#[get("/{username}/{playlist_name}/dislike")]
pub async fn playlist_dislike(
    username: Path<String>,
    playlist_name: Path<String>,
    claims: Claims,
) -> impl Responder {
    let playlist_name = playlist_name.to_string();
    let username = username.to_string();
    let mut db = fetch_db!();
    let Some(u) = User::from_id(&mut db, &claims.sub).await else {
        return HttpResponse::Forbidden();
    };
    // if they are requesting themself
    let playlist = query_as!(
        Playlist,
        "select * from playlist where author = $1 and name = $2",
        username,
        playlist_name,
    )
    .fetch_optional(&mut db)
    .await;
    let Ok(Some(mut v)) = playlist else {
        return HttpResponse::BadRequest();
    };
    v.dislike(&claims.sub);
    if !(v.author_id == claims.sub
        || v.edit_list.contains(&claims.sub)
        || v.public_playlist
        || u.admin)
    {
        return HttpResponse::Forbidden();
    }
    if query!(
        "update playlist set likes = $1 where author = $2 and name = $3",
        &v.likes,
        username,
        playlist_name
    )
    .execute(&mut db)
    .await
    .is_ok()
    {
        HttpResponse::Ok()
    } else {
        HttpResponse::Forbidden()
    }
}

#[get("/{username}/{playlist_name}/remove")]
pub async fn playlist_remove(
    req: HttpRequest,
    username: Path<String>,
    playlist_name: Path<String>,
    claims: Claims,
) -> impl Responder {
    let Some(songs_to_remove) = req.headers().get("songs") else {
        return HttpResponse::BadRequest();
    };
    let Ok(songs_to_remove) = songs_to_remove.to_str() else {
        return HttpResponse::BadRequest();
    };
    let Ok(songs_to_remove): Result<Vec<String>, _> = serde_json::from_str(songs_to_remove) else {
        return HttpResponse::BadRequest();
    };
    let playlist_name = playlist_name.to_string();
    let username = username.to_string();
    let mut db = fetch_db!();
    let Some(_u) = User::from_id(&mut db, &claims.sub).await else {
        return HttpResponse::Forbidden();
    };
    let playlist = query_as!(
        Playlist,
        "select * from playlist where author = $1 and name = $2",
        username,
        playlist_name,
    )
    .fetch_optional(&mut db)
    .await;
    let Ok(Some(mut v)) = playlist else {
        return HttpResponse::BadRequest();
    };
    v.songs.retain(|x| !songs_to_remove.contains(x));
    if query!(
        "update playlist set songs = $1 where author = $2 and name = $3",
        &v.songs,
        username,
        playlist_name
    )
    .execute(&mut db)
    .await
    .is_ok()
    {
        HttpResponse::Ok()
    } else {
        HttpResponse::BadRequest()
    }
}

#[get("/{username}/{playlist_name}/add")]
pub async fn playlist_add(
    req: HttpRequest,
    username: Path<String>,
    playlist_name: Path<String>,
    claims: Claims,
) -> impl Responder {
    let Some(songs_to_add) = req.headers().get("songs") else {
        return HttpResponse::BadRequest();
    };
    let Ok(songs_to_add) = songs_to_add.to_str() else {
        return HttpResponse::BadRequest();
    };
    let Ok(mut songs_to_add): Result<Vec<String>, _> = serde_json::from_str(songs_to_add) else {
        return HttpResponse::BadRequest();
    };
    let playlist_name = playlist_name.to_string();
    let username = username.to_string();
    let mut db = fetch_db!();
    let Some(_u) = User::from_id(&mut db, &claims.sub).await else {
        return HttpResponse::Forbidden();
    };
    let playlist = query_as!(
        Playlist,
        "select * from playlist where author = $1 and name = $2",
        username,
        playlist_name,
    )
    .fetch_optional(&mut db)
    .await;
    let Ok(Some(mut v)) = playlist else {
        return HttpResponse::BadRequest();
    };
    v.songs.append(&mut songs_to_add);
    if query!(
        "update playlist set songs = $1 where author = $2 and name = $3",
        &v.songs,
        username,
        playlist_name
    )
    .execute(&mut db)
    .await
    .is_ok()
    {
        HttpResponse::Ok()
    } else {
        HttpResponse::BadRequest()
    }
}

#[get("/{username}/{playlist_name}/delete")]
pub async fn playlist_delete(
    username: Path<String>,
    playlist_name: Path<String>,
    claims: Claims,
) -> impl Responder {
    let playlist_name = playlist_name.to_string();
    let username = username.to_string();
    let mut db = fetch_db!();
    let Some(u) = User::from_id(&mut db, &claims.sub).await else {
        return HttpResponse::Forbidden();
    };
    // if they are requesting themself
    let playlist = query_as!(
        Playlist,
        "select * from playlist where author = $1 and name = $2",
        username,
        playlist_name,
    )
    .fetch_optional(&mut db)
    .await;
    let Ok(Some(v)) = playlist else {
        return HttpResponse::BadRequest();
    };
    if v.author_id == claims.sub || u.admin {
        return match query!(
            "delete from playlist where author = $1 and name = $2",
            username,
            playlist_name
        )
        .execute(&mut db)
        .await
        {
            Ok(_) => HttpResponse::Ok(),
            _ => HttpResponse::Forbidden(),
        };
    }
    HttpResponse::BadRequest()
}

#[get("/{username}/{playlist_name}/edit")]
pub async fn playlist_edit(
    username: Path<String>,
    playlist_name: Path<String>,
    req: HttpRequest,
    claims: Claims,
) -> impl Responder {
    let mut db = fetch_db!();
    let username = username.to_string();
    let (Some(u), Some(d)) = (
        User::from_id(&mut db, &claims.sub).await,
        req.headers().get("data"),
    ) else {
        return HttpResponse::Forbidden();
    };
    let playlist_name = playlist_name.to_string();
    // if they are requesting themself
    let playlist = query_as!(
        Playlist,
        "select * from playlist where author = $1 and name = $2",
        username,
        playlist_name,
    )
    .fetch_all(&mut db)
    .await;
    let d = d.to_str().unwrap_or_default();
    let (Ok(v), Ok(d)) = (playlist, serde_json::from_str(d)) else {
        return HttpResponse::BadRequest();
    };
    let mut v: Vec<Playlist> = v
        .into_iter()
        .filter_map(|x| {
            if x.author_id == claims.sub
                || x.edit_list.contains(&username)
                || x.public_playlist
                || u.admin
            {
                Some(x)
            } else {
                None
            }
        })
        .collect();
    let Some(v) = v.first_mut() else {
        return HttpResponse::BadRequest();
    };
    let v = Playlist::update(v, d);
    if Playlist::update_playlist(db, v).await.is_ok() {
        HttpResponse::Ok()
    } else {
        HttpResponse::BadRequest()
    }
}
