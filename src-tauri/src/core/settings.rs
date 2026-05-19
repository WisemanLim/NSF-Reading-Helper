use crate::core::db::Db;
use crate::core::error::AppResult;
use crate::core::models::Settings;

const KEY: &str = "user_settings_v1";

pub fn load(db: &Db) -> AppResult<Settings> {
    match db.settings_get(KEY)? {
        Some(s) => Ok(serde_json::from_str(&s).unwrap_or_default()),
        None => Ok(Settings::default()),
    }
}

pub fn save(db: &Db, settings: &Settings) -> AppResult<()> {
    let s = serde_json::to_string(settings)?;
    db.settings_set(KEY, &s)
}
