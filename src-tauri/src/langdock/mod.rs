mod client;
mod profiles;

pub use client::{ConnectivityOutcome, LangDockClient};
pub use profiles::{
    ConnectionStatus, ConnectionWidgetState, LangdockProfile, ProfileInput, ProfileService,
    normalize_base_url,
};
