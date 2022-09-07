use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// The number of songs that we report back with last played
const LAST_PLAYED_LENGTH: usize = 20;

#[derive(Deserialize)]
pub(crate) struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    pub port: u16,
    pub client_origin_url: String,
    max_songs: usize,
    max_song_folder_size_gb: usize,
    retries: usize,
    yt_timeout_ms: usize,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
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

pub(crate) struct User {
    id: String,
    username: String,

    // do we want to store preferences on the server
    store_data_serverside: bool,
    // store preferences for sending thumbnail data for low speed connections
    thumbnails: bool,
    // auto play songs when done
    autoplay: bool,

    // allow people to follow
    allow_followers: bool,
    // public profile that can be searched
    public: bool,
    // show song activity
    activity: bool,
    // show last played
    last_played: VecDeque<String>,
    // display nick name
    display_name: String,
    // id of followers
    followers: Vec<String>,
    // id of following
    following: Vec<String>,
    // count to playback statistics
    analytics: bool,
}

impl User {
    pub(crate) fn now_playing(&mut self, id: String) {
        if self.last_played.len() > 19 {
            let _ = self.last_played.pop_back();
        }
        self.last_played.push_front(id);
    }
    // pub(crate) fn update_user(&mut self, jjjjjjj)
}

pub(crate) struct Song {
    id: String,
    title: String,
    album: Option<String>,
    artist: String,
    duration: f64,
    genre: Option<String>,
    track_disc: [u16; 2],
    album_arist: Vec<String>,
    size: u64,
}
