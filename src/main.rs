use api::{create_post, delete_post, latest, read_post, update_post};
use axum::{
    routing::{delete, get, post, put},
    serve, Router,
};
use config::Config;
#[cfg(feature = "update_cache")]
use reqwest::Client;

mod api;
mod config;
mod error;
#[derive(Clone)]
struct AppState {
    config: Config,
    #[cfg(feature = "update_cache")]
    client: Client,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let config: Config = confy::load_path("/etc/blog-markdown-rs/config.toml")?;
    let state = AppState {
        config,
        #[cfg(feature = "update_cache")]
        client: Client::new(),
    };
    let listener = tokio::net::TcpListener::bind(state.config.listen).await?;
    serve(listener, router(state)).await?;
    Ok(())
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/post", post(create_post))
        .route("/post/:id", get(read_post))
        .route("/post/:id", put(update_post))
        .route("/post/:id", delete(delete_post))
        .route("/post/latest", get(latest))
        // index is generated at each request, so it is suggested to use a caching proxy that will get invalidated when write operation are done on posts (post,put,delete)
        // .route("/index", get(index))
        // same here, using a caching proxy is recommended.
        // .route("/rss", get(rss))
        .with_state(state)
}
