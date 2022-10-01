use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
};

#[derive(Deserialize, Serialize)]
pub(crate) struct VideoData {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) thumbnail: String,
    pub(crate) uploader: String,
    pub(crate) uploader_url: String,
    pub(crate) duration: usize,
    pub(crate) age_limit: i64,
    pub(crate) webpage_url: String,
    pub(crate) was_live: bool,
    pub(crate) upload_date: String,
    pub(crate) filesize: i64,
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
    pub(crate) fn load_and_replace(id: &str) -> Result<Self> {
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
