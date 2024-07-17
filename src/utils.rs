use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use axum::extract::Request;
use chrono::{Locale, NaiveDateTime};
use reqwest::header::ACCEPT_LANGUAGE;

cfg_if::cfg_if! {
    if #[cfg(feature="update_cache")] {
use axum::http::HeaderValue;
use get_pass::get_password;
use reqwest::{header::HOST, IntoUrl};
    }
}
use crate::{error::AppError, AppState};

pub fn detect_title_md(md: &Path) -> Result<Option<String>, AppError> {
    let contents = std::fs::read_to_string(md)?;
    for line in contents.lines() {
        if line.starts_with("# ") {
            let title = line.replace("# ", "");
            return Ok(Some(title));
        }
    }
    Ok(None)
}
pub fn detect_language(request: &Request) -> Locale {
    if let Some(header_value) = request.headers().get(ACCEPT_LANGUAGE) {
        if let Ok(str) = header_value.to_str() {
            let vec = accept_language::parse(str);
            if !vec.is_empty() {
                if let Ok(value) = Locale::from_str(&vec[0]) {
                    return value;
                }
            }
        }
    }
    // if header is not present/invalid, fallback to english
    Locale::en_GB
}
pub fn path_markdown(state: &AppState, id: &NaiveDateTime) -> PathBuf {
    let mut path = state.config.assets_path.to_owned();
    let name = format!("{id}.md");
    path.push(name);
    path
}
#[cfg(feature = "update_cache")]
pub fn uri_with_pass(state: &AppState, path: &str) -> url::Url {
    let mut url = state.config.cache_uri.to_owned();

    if let Some(pass_path) = &state.config.cache_password {
        let password = get_password(pass_path).unwrap();
        url.set_password(Some(&password)).unwrap();
    }
    url.join(path).unwrap();
    url
}
#[cfg(feature = "update_cache")]
pub async fn invalidate_cache(state: AppState, url: impl IntoUrl) {
    state
        .client
        .delete(url)
        .header(HOST, HeaderValue::from_str(&state.config.hostname).unwrap())
        .send()
        .await
        .unwrap();
}
