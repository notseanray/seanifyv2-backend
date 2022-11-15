use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
};

#[derive(Deserialize, Serialize)]
pub struct VideoData {
    pub id: String,
    pub title: String,
    pub thumbnail: String,
    pub uploader: String,
    pub uploader_url: String,
    pub duration: usize,
    pub age_limit: i64,
    pub webpage_url: String,
    pub was_live: bool,
    pub upload_date: String,
    pub filesize: i64,
}

// #[derive(Debug, Display)]
// enum YTError {
//     #[display(fmt = "metadata extraction failure")]
//     MetadataConversionFailure,
// }

impl VideoData {
    fn from_yt_file(id: &str) -> Result<Self> {
        let data = fs::read_to_string(format!("songs/{id}.info.json"))?;
        Ok(serde_json::from_str(&data)?)
    }
    pub fn load_and_replace(id: &str) -> Result<Self> {
        let full = Self::from_yt_file(id)?;
        let d = serde_json::to_string(&full)?;
        let mut f = File::options()
            .write(true)
            .open(format!("songs/{id}.info.json"))?;
        f.set_len(d.len() as u64)?;
        f.write_all(d.as_bytes())?;
        Ok(full)
    }
}
