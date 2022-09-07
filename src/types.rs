use serde::{Deserialize, Serialize};

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
    username: String,
    password: String,
    public: bool,
    last_played: VecDeque<String>,
    display_name: String,
    followers: Vec<String>,
    following: Vec<String>,
    analytics: bool,
    admin: bool,
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
