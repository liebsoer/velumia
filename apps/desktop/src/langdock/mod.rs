mod client;
mod completions;
mod profiles;

pub use client::{ConnectivityOutcome, LangDockClient};
pub use completions::{
    build_completion_request, parse_vercel_text_delta_line, CompletionAgent,
    CompletionMessage, CompletionRequestBody, CompletionStreamError,
};
pub use profiles::{
    ConnectionStatus, ConnectionWidgetState, LangdockProfile, ProfileInput, ProfileService,
    normalize_base_url,
};
