use crate::{
    fuzzy::{fuzzy_search_best_n, SearchType},
    CONFIG,
};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fs, process::Command};

// The number of songs that we report back with last played
// const LAST_PLAYED_LENGTH: usize = 30;
const MAX_LAST_PLAYED: usize = 30;
const MAX_SEARCH_RESULTS: usize = 30;

#[derive(Deserialize)]
pub(crate) struct Config {
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
pub(crate) struct ErrorMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    pub message: String,
}

#[derive(Serialize)]
pub(crate) struct User {
    pub(crate) id: String,
    pub(crate) username: String,

    // do we want to store preferences on the server
    pub(crate) serverside: bool,
    // store preferences for sending thumbnail data for low speed connections
    pub(crate) thumbnails: bool,
    // auto play songs when done
    pub(crate) autoplay: bool,

    // allow people to follow
    pub(crate) allow_followers: bool,
    // public profile that can be searched
    pub(crate) public_account: bool,
    // show song activity
    pub(crate) activity: bool,
    // show last played
    pub(crate) last_played: VecDeque<String>,
    // display nick name
    pub(crate) display_name: String,
    // id of followers
    pub(crate) followers: Vec<String>,
    // id of following
    pub(crate) following: Vec<String>,
    // count to playback statistics
    pub(crate) analytics: bool,
}

impl User {
    pub(crate) fn now_playing(&mut self, id: String) {
        if self.last_played.len() > CONFIG.max_last_played {
            let _ = self.last_played.pop_back();
        }
        self.last_played.push_front(id);
    }
    // pub(crate) fn update_user(&mut self, jjjjjjj)
}

#[derive(Serialize)]
pub(crate) struct UserFromDB {
    pub(crate) id: String,
    pub(crate) username: String,

    // do we want to store preferences on the server
    pub(crate) serverside: bool,
    // store preferences for sending thumbnail data for low speed connections
    pub(crate) thumbnails: bool,
    // auto play songs when done
    pub(crate) autoplay: bool,

    // allow people to follow
    pub(crate) allow_followers: bool,
    // public profile that can be searched
    pub(crate) public_account: bool,
    // show song activity
    pub(crate) activity: bool,
    // show last played
    pub(crate) last_played: String,
    // display nick name
    pub(crate) display_name: String,
    // id of followers
    pub(crate) followers: String,
    // id of following
    pub(crate) following: String,
    // count to playback statistics
    pub(crate) analytics: bool,
}

impl From<UserFromDB> for User {
    fn from(u: UserFromDB) -> Self {
        Self {
            id: u.id,
            username: u.username,
            serverside: u.serverside,
            thumbnails: u.thumbnails,
            autoplay: u.autoplay,
            allow_followers: u.allow_followers,
            public_account: u.public_account,
            activity: u.activity,
            last_played: u.last_played.split('`').map(|x| x.into()).collect(),
            display_name: u.display_name,
            followers: u.followers.split('`').map(|x| x.into()).collect(),
            following: u.following.split('`').map(|x| x.into()).collect(),
            analytics: u.analytics,
        }
    }
}

pub(crate) struct Song<'a> {
    pub(crate) id: &'a str,
    pub(crate) title: &'a str,
    pub(crate) album: Option<&'a str>,
    pub(crate) artist: &'a str,
    pub(crate) duration: f64,
    pub(crate) genre: Option<&'a str>,
    pub(crate) track_disc: [u16; 2],
    pub(crate) album_arist: Vec<&'a str>,
    pub(crate) size: u64,
    pub(crate) default_search: &'a str,
}

impl Song<'_> {
    fn get_id(url: &str) -> Option<&str> {
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
    pub(crate) async fn from_url(url: &str) -> Option<Song<'_>> {
        if let Some(v) = Self::get_id(url) {
            let _ = fs::create_dir_all("./songs");
            let mut cmd = Command::new("yt-dlp");
            cmd.args([
                "--socket-timeout",
                &CONFIG.yt_timeout_sec,
                "--embed-thumbnail",
                "--audio-format",
                "mp3",
                "--embed-thumbnail",
                "--extract-audio",
                "--output",
                &format!("songs/{v}"),
                url,
            ]);
            let cmd = cmd.output().expect("yt dlp not installed");
            // insert into db
        }
        unimplemented!();
    }
    pub(crate) async fn insert() {}
}

pub(crate) struct SongSearch<'a> {
    songs: &'a Vec<Song<'a>>,
}

impl SongSearch<'_> {
    pub(crate) fn load() {}
    pub(crate) fn search<'a>(
        &self,
        term: &'a str,
        search_type: SearchType,
        amount: usize,
    ) -> Vec<(&'a Song, f32)> {
        fuzzy_search_best_n(term, self.songs, amount, &search_type)
    }
}

// pub(crate) struct Playlist<'a> {
//
// }
