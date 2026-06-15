use crate::db::AppDatabase;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<AppDatabase>,
}

impl AppState {
    pub fn with_db<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&AppDatabase) -> Result<T, String>,
    {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        f(&db)
    }
}

pub fn principal(db: &AppDatabase) -> Result<crate::authz::Principal, String> {
    Ok(crate::authz::Principal {
        user_id: db.solo_user_id().map_err(|e| e.to_string())?,
    })
}
