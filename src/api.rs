pub mod playlist;
pub mod routes;
pub mod songs;
mod types;
pub mod users;

#[macro_export]
macro_rules! fetch_db {
    () => {
        (*DB.get().await).db.try_acquire().unwrap()
    };
}

#[macro_export]
macro_rules! response {
    ($message:expr) => {
        web::Json(Message {
            metadata: Metadata {
                api: VERSION.to_string(),
                branch: BRANCH.to_string(),
            },
            text: $message.to_string(),
        })
    };
}

#[macro_export]
macro_rules! time {
    () => {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
    };
}

pub struct BoolResult {
    admin: bool,
}
