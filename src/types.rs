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

#[derive(Serialize)]
pub(crate) struct User {
    id: String,
    username: String,

    // do we want to store preferences on the server
    serverside: bool,
    // store preferences for sending thumbnail data for low speed connections
    thumbnails: bool,
    // auto play songs when done
    autoplay: bool,

    // allow people to follow
    allow_followers: bool,
    // public profile that can be searched
    public_account: bool,
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
            analytics: u.analytics
        }
    }
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
