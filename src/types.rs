use crate::{
    fetch_db,
    fuzzy::{fuzzy_search_best_n, SearchType},
    youtube::VideoData,
    CONFIG, DB, SONG_SEARCH,
};
use anyhow::{anyhow, Result};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, query, query_as, Postgres};
use std::future::Future;
use std::{collections::VecDeque, fs, process::Command};

// The number of songs that we report back with last played
// const LAST_PLAYED_LENGTH: usize = 30;
pub(crate) const MAX_LAST_PLAYED: usize = 30;
pub(crate) const MAX_SEARCH_RESULTS: usize = 30;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_client_origin_url")]
    pub client_origin_url: String,
    #[serde(default = "default_max_last_played")]
    pub max_last_played: usize,
    #[serde(default = "default_max_search_results")]
    pub max_search_results: usize,
    #[serde(default = "default_max_songs")]
    pub max_songs: usize,
    #[serde(default = "default_max_song_folder_size_gb")]
    pub max_song_folder_size_gb: usize,
    #[serde(default = "default_max_retries")]
    pub retries: usize,
    #[serde(default = "default_yt_timeout_sec")]
    pub yt_timeout_sec: String,
}

fn default_host() -> String {
    String::from("127.0.0.1")
}

fn default_port() -> u16 {
    6000
}

fn default_client_origin_url() -> String {
    String::from("http://localhost:6000")
}

fn default_max_last_played() -> usize {
    20
}

fn default_max_search_results() -> usize {
    30
}

fn default_max_songs() -> usize {
    10000
}

fn default_max_song_folder_size_gb() -> usize {
    10
}

fn default_max_retries() -> usize {
    3
}

fn default_yt_timeout_sec() -> String {
    String::from("3")
}

impl Default for Config {
    fn default() -> Self {
        envy::from_env::<Config>().expect("Provide missing environment variables for Config")
    }
}

#[derive(Serialize)]
pub struct ErrorMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,

    // do we want to store preferences on the server
    pub serverside: bool,
    // store preferences for sending thumbnail data for low speed connections
    pub thumbnails: bool,
    // auto play songs when done
    pub autoplay: bool,

    // allow people to follow
    pub allow_followers: bool,
    // public profile that can be searched
    pub public_account: bool,
    // show song activity
    pub activity: bool,
    // show last played
    pub last_played: Vec<String>,
    // display nick name
    pub display_name: String,
    // id of followers
    pub followers: Vec<String>,
    // id of following
    pub following: Vec<String>,
    // ids of liked songs
    pub likes: Vec<String>,
    // ids of liked playlists
    pub liked_playlist: Vec<String>,
    // count to playback statistics
    pub analytics: bool,
    pub admin: bool,
    pub lastupdate: String,
}

// no need to convert structs just to do a tiny operation
impl User {
    pub fn now_playing(&mut self, new_song: String) -> &Vec<String> {
        if self.last_played.len() > MAX_LAST_PLAYED {
            self.last_played.remove(0);
        }
        self.last_played.push(new_song);
        &self.last_played
    }
    pub fn follow(&mut self, follower: String) {
        self.followers.push(follower);
    }
    pub fn unfollow(&mut self, unfollower: &str) {
        self.followers.retain(|x| x.as_str() != unfollower)
    }
    pub async fn from_id(db: &mut PoolConnection<Postgres>, id: &str) -> Option<Self> {
        if let Ok(v) = query_as!(User, "select * from users where id = $1", id)
            .fetch_optional(db)
            .await
        {
            return v;
        }
        None
    }

    pub async fn from_username(db: &mut PoolConnection<Postgres>, username: &str) -> Option<Self> {
        if let Ok(v) = query_as!(User, "select * from users where username = $1", username)
            .fetch_optional(db)
            .await
        {
            return v;
        }
        None
    }

    pub fn like(&mut self, id: String) {
        self.likes.push(id);
    }
    pub fn dislike(&mut self, id: &str) {
        self.likes.retain(|x| x.as_str() != id)
    }
}

#[derive(Default)]
pub struct DownloadCache(VecDeque<(String, String)>);

impl DownloadCache {
    pub fn append(&mut self, url: String, user: String) {
        self.0.push_front((url, user));
    }
    pub async fn cycle(&mut self, db: &mut PoolConnection<Postgres>) {
        if let Some((url, user)) = self.0.pop_back() {
            // TODO ws broadcast
            Song::from_url(&url, db, user).await;
        }
    }
    pub fn clear(&mut self) {
        self.0.clear();
    }
    pub fn list(&self) -> String {
        serde_json::to_string(&self.0).unwrap()
    }
}

#[derive(Serialize, Clone)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub uploader: String,
    pub artist: String,
    pub genre: String,
    pub album: String,
    pub url: String,
    pub duration: f64,
    pub age_limit: i32,
    pub webpage_url: String,
    pub was_live: bool,
    pub upload_date: String,
    pub filesize: i64,
    pub added_by: String,
    pub default_search: String,
}

#[derive(Debug, Display)]
enum SongError {
    #[display(fmt = "metadata extraction failure")]
    MetadataExtractionFailure,
}

type SE = SongError;
impl<'a> Song {
    // convert to return Result<Error>
    fn get_id(url: &'a str) -> Option<&'a str> {
        let id = url.find("?v=");
        if let Some(v) = id {
            let split = &url[v..];
            let end = split.find('?');
            return Some(if let Some(v) = end {
                &split[..v]
            } else {
                split
            });
        }
        None
    }
    // yt-dlp --socket-timeout 3 --embed-thumbnail --audio-format mp3 --extract-audio --output "M3HhNcl2dMA.%(ext)s" --add-metadata --write-info-json https://www.youtube.com/watch\?v\=M3HhNcl2dMA
    pub async fn from_url(
        url: &'a str,
        db: &mut PoolConnection<Postgres>,
        user: String,
    ) -> Option<Song> {
        if let Some(v) = Self::get_id(url) {
            let _ = fs::create_dir_all("./songs");
            let mut cmd = Command::new("yt-dlp");
            cmd.args([
                "--socket-timeout",
                &CONFIG.yt_timeout_sec,
                "--embed-thumbnail",
                "--audio-format",
                "mp3",
                "--retries",
                &CONFIG.retries.to_string(),
                "--extract-audio",
                "--add-metadata",
                "--output",
                &format!("songs/{v}.%(ext)s"),
                "--write-info-json",
                url,
            ]);
            let cmd = cmd.output().expect("yt dlp not installed");
            if let (true, Ok(s)) = (
                cmd.status.success(),
                Self::insert(v.to_string(), db, url, user).await,
            ) {
                // ws msg
                let mut db = fetch_db!();
                SONG_SEARCH.get().await.write().await.update(&mut db).await;
                None
            } else {
                None
            }
        } else {
            None
        }
    }
    // pass in db handle from from_url
    async fn insert(
        id: String,
        db: &mut PoolConnection<Postgres>,
        url: &str,
        user_id: String,
    ) -> Result<Song> {
        let meta = match mp3_metadata::read_from_file(format!("songs/{id}.mp3")) {
            Ok(v) => v,
            _ => return Err(anyhow!("Failed to extract metadata")),
        };
        let data = VideoData::load_and_replace(&id)?;
        if let Some(tag) = meta.tag {
            let artist = if tag.artist.is_empty() {
                data.uploader.clone()
            } else {
                tag.artist.clone()
            };
            let new_song = Self {
                default_search: format!("{} {} {}", &data.title, &tag.artist, &tag.album),
                id,
                title: data.title,
                uploader: data.uploader,
                url: url.to_string(),
                artist,
                genre: format!("{:?}", tag.genre),
                album: tag.album,
                duration: meta.duration.as_secs_f64(),
                age_limit: data.age_limit as i32,
                webpage_url: data.webpage_url,
                was_live: data.was_live,
                upload_date: data.upload_date,
                filesize: data.filesize,
                added_by: user_id,
            };
            let _ = query!(
                r#"insert into songs(
                    id,
                    title,
                    uploader,
                    artist,
                    genre,
                    album,
                    url,
                    duration,
                    age_limit,
                    webpage_url,
                    was_live,
                    upload_date,
                    filesize,
                    added_by,
                    default_search)
                values($1,
                       $2,
                       $3,
                       $4,
                       $5,
                       $6,
                       $7,
                       $8,
                       $9,
                       $10,
                       $11,
                       $12,
                       $13,
                       $14,
                   $15)"#,
                new_song.id,
                new_song.title,
                new_song.uploader,
                new_song.artist,
                new_song.genre,
                new_song.album,
                new_song.url,
                new_song.duration,
                new_song.age_limit as i32,
                new_song.webpage_url,
                new_song.was_live,
                new_song.upload_date,
                new_song.filesize as i64,
                new_song.added_by,
                new_song.default_search
            )
            .execute(db)
            .await;
            Ok(new_song)
        } else {
            Err(anyhow!("failed to read song data"))
        }
    }
}

pub struct SongSearch {
    songs: Vec<Song>,
}

impl SongSearch {
    pub async fn load(db: &mut PoolConnection<Postgres>) -> Self {
        let songs: Vec<Song> = query_as!(Song, "select * from songs")
            .fetch_all(db)
            .await
            .unwrap();
        Self { songs }
    }
    pub async fn update(&mut self, db: &mut PoolConnection<Postgres>) {
        let songs: Vec<Song> = query_as!(Song, "select * from songs")
            .fetch_all(db)
            .await
            .unwrap();
        self.songs = songs;
    }

    #[inline]
    pub fn search(&self, term: &str, search_type: SearchType, amount: usize) -> Vec<(&Song, f32)> {
        fuzzy_search_best_n(term, &self.songs, amount, &search_type)
    }

    pub fn get_by_id(&self, id: &str) -> Option<Song> {
        // TODO switch to rayon
        let songs: Vec<Song> = self
            .songs
            .iter()
            .filter(|x| Some(x.id.as_str()) == Some(id))
            .cloned()
            .collect();
        songs.first().cloned()
    }
}

#[derive(Deserialize)]
pub struct PlaylistEditable {
    name: String,
    public_playlist: bool,
    songs: Vec<String>,
    description: String,
    cover: String,
}

impl Playlist {
    pub fn like(&mut self, like: String) {
        self.likes.push(like)
    }
    pub fn dislike(&mut self, disliker: &str) {
        self.likes.retain(|x| x.as_str() != disliker)
    }
    pub fn update(playlist: &mut Self, data: PlaylistEditable) -> &mut Self {
        playlist.name = data.name;
        playlist.public_playlist = data.public_playlist;
        playlist.songs = data.songs;
        playlist.description = data.description;
        playlist.cover = data.cover;
        playlist
    }
    pub async fn update_playlist(
        mut db: PoolConnection<Postgres>,
        data: &mut Self,
    ) -> Result<(), PlaylistError> {
        let current_playlist = match query_as!(
            Playlist,
            "select * from playlist where author = $1 and name = $2",
            data.author,
            data.name
        )
        .fetch_optional(&mut db)
        .await
        {
            Ok(Some(v)) => v,
            _ => return Err(PlaylistError::NotExist),
        };
        let mut new_duration: f64 = 0.0;
        if current_playlist.songs != data.songs {
            for song in data.songs.iter() {
                if let Some(song) = SONG_SEARCH
                    .get()
                    .await
                    .write()
                    .await
                    .get_by_id(song.as_str())
                {
                    new_duration += song.duration;
                } else {
                    return Err(PlaylistError::InvalidSong);
                }
            }
        }
        data.duration = (new_duration + 0.5) as i64;
        // let current_playlist: Playlist = current_playlist.into();
        let result = query!(
            r#"update playlist set
                public_playlist = $1,
                songs = $2,
                description = $3,
                likes = $4,
                cover = $5,
                duration = $6,
                lastupdate = $7
            where
                name = $8 and
                author = $9"#,
            data.public_playlist,
            &data.songs,
            data.description,
            &data.likes,
            data.cover,
            data.duration,
            data.lastupdate,
            data.name,
            data.author
        )
        .execute(&mut db)
        .await;
        if result.is_ok() {
            Ok(())
        } else {
            Err(PlaylistError::InvalidData)
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Playlist {
    pub name: String,
    pub public_playlist: bool,
    pub songs: Vec<String>,
    pub author: String,
    pub author_id: String,
    pub edit_list: Vec<String>,
    pub description: String,
    pub likes: Vec<String>,
    pub cover: String,
    pub duration: i64,
    pub lastupdate: String,
}

pub enum PlaylistError {
    NotExist,
    InvalidData,
    InvalidSong,
}
