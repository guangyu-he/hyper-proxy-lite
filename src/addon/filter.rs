use anyhow::Result;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use std::collections::HashSet;

pub enum FilterMode {
    Blacklist,
    Whitelist,
}
pub struct FilterRules {
    mode: FilterMode,
    domains: HashSet<String>,
}

impl FilterRules {
    #[allow(dead_code)]
    pub fn new_blacklist<S: Into<String>>(domains: Vec<S>) -> Self {
        FilterRules {
            mode: FilterMode::Blacklist,
            domains: domains.into_iter().map(|s| s.into()).collect(),
        }
    }

    #[allow(dead_code)]
    pub fn new_whitelist<S: Into<String>>(domains: Vec<S>) -> Self {
        FilterRules {
            mode: FilterMode::Whitelist,
            domains: domains.into_iter().map(|s| s.into()).collect(),
        }
    }

    pub fn is_allowed(&self, host: &str) -> bool {
        let domain = host.split(':').next().unwrap_or(host).to_string();

        match self.mode {
            FilterMode::Blacklist => !self.domains.contains(&domain),
            FilterMode::Whitelist => self.domains.contains(&domain),
        }
    }
}

/// Generate a blocked response for a given host.
/// This function creates an HTTP 403 Forbidden response
/// with a message indicating that access to the specified host
/// is blocked by proxy filter rules.
pub fn blocked_response(host: &str) -> Result<Response<BoxBody<Bytes, hyper::Error>>> {
    let body = format!("Access to {} is blocked by proxy filter rules", host);

    let response = Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("Content-Type", "text/plain")
        .body(
            Full::new(Bytes::from(body))
                .map_err(|never| match never {})
                .boxed(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to build blocked response: {}", e))?;

    Ok(response)
}
