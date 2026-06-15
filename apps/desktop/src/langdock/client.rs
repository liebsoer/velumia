use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectivityOutcome {
    Connected,
    ConfigurationError,
    Offline,
    RateLimited,
}

pub struct LangDockClient {
    http: reqwest::Client,
}

impl LangDockClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("http client");
        Self { http }
    }

    pub async fn probe_models(&self, base_url: &str, api_key: &str) -> ConnectivityOutcome {
        let url = format!(
            "{}/agent/v1/models",
            base_url.trim_end_matches('/')
        );

        let mut last_outcome = ConnectivityOutcome::Offline;

        for attempt in 0..2 {
            match self.http.get(&url).bearer_auth(api_key).send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    return match status {
                        200 => ConnectivityOutcome::Connected,
                        401 | 403 => ConnectivityOutcome::ConfigurationError,
                        429 => ConnectivityOutcome::RateLimited,
                        _ if status >= 500 => {
                            last_outcome = ConnectivityOutcome::Offline;
                            if attempt == 0 {
                                continue;
                            }
                            ConnectivityOutcome::Offline
                        }
                        _ => ConnectivityOutcome::ConfigurationError,
                    };
                }
                Err(_) => {
                    last_outcome = ConnectivityOutcome::Offline;
                    if attempt == 0 {
                        continue;
                    }
                }
            }
        }

        last_outcome
    }
}

impl Default for LangDockClient {
    fn default() -> Self {
        Self::new()
    }
}
