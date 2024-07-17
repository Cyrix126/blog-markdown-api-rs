use crate::index::generate_index;
use std::{fs::DirEntry, path::PathBuf, str::FromStr};

cfg_if::cfg_if! {
    if #[cfg(feature="update_cache")] {
use axum::http::HeaderValue;
use tokio::spawn;
use get_pass::get_password;
use reqwest::{header::HOST, IntoUrl};
    }
}
use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{Local, Locale, NaiveDateTime};
use reqwest::header::ACCEPT_LANGUAGE;
use tokio::{
    fs::{read_to_string, remove_file, File},
    io::AsyncWriteExt,
};

use crate::{error::AppError, AppState};

pub async fn create_post(
    State(state): State<AppState>,
    // check that the body is valid utf-8
    body: String,
) -> Result<impl IntoResponse, AppError> {
    let datetime = Local::now().naive_utc();
    let path = path_markdown(&state, &datetime);
    // save body as file in path with name datetime
    let mut output = File::create(path).await?;
    output.write_all(body.as_bytes()).await?;
    // Return id of post (the datetime) in body
    // update the cache of latest
    #[cfg(feature = "update_cache")]
    spawn(enclose::enc!((state) async move {
    let path = "/post/latest".to_string();
    let url = uri_with_pass(&state, &path);
        invalidate_cache(state, url).await;
    }));
    Ok((StatusCode::CREATED, datetime.to_string()))
}
pub async fn read_post(
    State(state): State<AppState>,
    Path(id): Path<NaiveDateTime>,
) -> Result<impl IntoResponse, AppError> {
    let path = path_markdown(&state, &id);
    let body = read_to_string(path).await.map_err(|_| AppError::NotFound)?;
    Ok(body)
}
pub async fn update_post(
    State(state): State<AppState>,
    // check that the path is a valid datetime
    Path(id): Path<NaiveDateTime>,
    body: String,
) -> Result<impl IntoResponse, AppError> {
    let path = path_markdown(&state, &id);
    // save body as file in path with name datetime
    let mut output = File::create(path).await?;
    output.write_all(body.as_bytes()).await?;
    // update cache since post is modified.
    #[cfg(feature = "update_cache")]
    spawn(enclose::enc!((state, id) async move {
    let path = format!("/post/{id}");
    let url = uri_with_pass(&state, &path);
        invalidate_cache(state, url).await;
    }));

    Ok(())
}
pub async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<NaiveDateTime>,
) -> Result<impl IntoResponse, AppError> {
    let path: PathBuf = path_markdown(&state, &id);
    remove_file(path).await?;
    // update cache since post is modified.
    #[cfg(feature = "update_cache")]
    spawn(enclose::enc!((state, id) async move {
    let path = format!("/post/{id}");
    let url = uri_with_pass(&state, &path);
        invalidate_cache(state, url).await;
    }));
    Ok(())
}

pub async fn index(
    State(state): State<AppState>,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    // detect accept language to use right translation of dates.
    let locale = detect_language(&request);
    // call function to generate index and return it.
    let index = generate_index(&state, locale)?;
    Ok(index)
}
// pub async fn rss(
//     State(state): State<AppState>,
//     request: Request,
// ) -> Result<impl IntoResponse, AppError> {
//     if let Some(lang) = request.headers().get(ACCEPT_LANGUAGE) {
//         //
//     }
//     todo!("");
//     // call function to generate rss and return it.
//     Ok(())
// }
pub async fn latest(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let mut paths: Vec<DirEntry> = std::fs::read_dir(state.config.assets_path)
        .unwrap()
        .map(|d| d.unwrap())
        .collect();
    paths.sort_unstable_by_key(|dir| dir.file_name());
    let path = paths.last().unwrap().path();
    let body = read_to_string(path).await?;
    Ok(body)
}

fn detect_language(request: &Request) -> Locale {
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

fn path_markdown(state: &AppState, id: &NaiveDateTime) -> PathBuf {
    let mut path = state.config.assets_path.to_owned();
    let name = format!("{id}.md");
    path.push(name);
    path
}
#[cfg(feature = "update_cache")]
fn uri_with_pass(state: &AppState, path: &str) -> url::Url {
    let mut url = state.config.cache_uri.to_owned();

    if let Some(pass_path) = &state.config.cache_password {
        let password = get_password(pass_path).unwrap();
        url.set_password(Some(&password)).unwrap();
    }
    url.join(path).unwrap();
    url
}
#[cfg(feature = "update_cache")]
async fn invalidate_cache(state: AppState, url: impl IntoUrl) {
    state
        .client
        .delete(url)
        .header(HOST, HeaderValue::from_str(&state.config.hostname).unwrap())
        .send()
        .await
        .unwrap();
}
