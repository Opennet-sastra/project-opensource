//! Sync coordinator: wires concrete `scraper::SastraPortalSource` into `storage`.
//! Depends on `domain`, `scraper`, `storage`, and `captcha`.

pub use domain::*;
use scraper::SastraPortalSource;
use storage::Storage;

#[derive(Clone)]
pub struct SyncService {
    storage: Storage,
    source: SastraPortalSource,
    ocr_endpoint: String,
}

impl Default for SyncService {
    fn default() -> Self {
        Self::new(Storage::new())
    }
}

impl SyncService {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage,
            source: SastraPortalSource::new(),
            ocr_endpoint: "http://localhost:8000/solve".to_string(),
        }
    }

    pub fn with_source(storage: Storage, source: SastraPortalSource) -> Self {
        Self {
            storage,
            source,
            ocr_endpoint: "http://localhost:8000/solve".to_string(),
        }
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub async fn fetch_captcha(&self) -> Result<CaptchaChallenge, String> {
        self.source.fetch_captcha().await.map_err(|e| e.to_string())
    }

    pub async fn try_solve_captcha(&self, data_uri: &str) -> Option<String> {
        captcha::try_solve_ocr(data_uri, &self.ocr_endpoint).await
    }

    pub async fn login_and_sync(
        &self,
        creds: &Credentials,
        captcha_code: &str,
        session_id: &str,
    ) -> Result<(), String> {
        let session = self
            .source
            .login(creds, captcha_code, session_id)
            .await
            .map_err(|e| e.to_string())?;

        self.storage.set_session(session.clone());

        let data = self
            .source
            .fetch_portal_data(&session)
            .await
            .map_err(|e| e.to_string())?;

        self.storage.set_portal_data(data);

        Ok(())
    }

    pub async fn refresh(&self) -> Result<(), String> {
        let session = self
            .storage
            .get_session()
            .ok_or_else(|| "No active session".to_string())?;

        let data = self
            .source
            .fetch_portal_data(&session)
            .await
            .map_err(|e| e.to_string())?;

        self.storage.set_portal_data(data);

        Ok(())
    }
}
