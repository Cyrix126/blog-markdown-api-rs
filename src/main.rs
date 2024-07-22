use api::{create_post, delete_post, latest, read_post, update_post};
use axum::{
    routing::{get, post},
    serve, Router,
};
use config::Config;
#[cfg(feature = "update_cache")]
use reqwest::Client;

mod api;
mod config;
mod error;
mod index;
mod rss;
mod utils;
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
        .route(
            "/post/:id",
            get(read_post).put(update_post).delete(delete_post),
        )
        .route("/post/latest", get(latest))
        // index is generated at each request, so it is suggested to use a caching proxy that will get invalidated when write operation are done on posts (post,put,delete)
        .route("/index", get(api::index))
        // same here, using a caching proxy is recommended.
        .route("/rss", get(api::rss))
        .with_state(state)
}

#[cfg(test)]
mod test {
    use axum_test::TestServer;
    use tempfile::TempDir;

    use crate::{config::Config, router, AppState};

    async fn app() -> Result<TestServer, Box<dyn std::error::Error>> {
        let tmp_dir = TempDir::new()?;
        let config = Config {
            assets_path: tmp_dir.into_path(),
            ..Default::default()
        };
        let state = AppState { config };
        let router = router(state);
        Ok(TestServer::new(router).unwrap())
    }
    // return a non existent post
    #[tokio::test]
    async fn request_404() -> Result<(), Box<dyn std::error::Error>> {
        let app = app().await?;
        app.get("/post/2015-09-18_23-56-04")
            .await
            .assert_status_not_found();
        Ok(())
    }
    // create a post
    // view the post
    // modify the post
    // view modified post
    // view index
    // view rss
    // delete post
}
