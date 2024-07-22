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
#[derive(Clone, Debug)]
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

#[cfg(not(feature = "update_cache"))]
#[cfg(test)]
mod test {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use tempfile::TempDir;

    use crate::{config::Config, router, AppState};
    const DOC_MARKDOWN: &str = r#"#Title
Text describing the markdown test
## A second title
**Bold description**"#;
    const DOC_MARKDOWN_UPDATED: &str = r#"#Title
Text describing the markdown test
## A second title
**Bold description**  
Another text."#;

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
    #[tokio::test]
    async fn crud() -> Result<(), Box<dyn std::error::Error>> {
        let app = app().await?;
        // create
        let rep = app.post("/post").text(DOC_MARKDOWN).await;
        rep.assert_status(StatusCode::CREATED);
        let id = rep.text();
        // read
        let path = ["/post/", &id].concat();
        let rep = app.get(&path).await;
        rep.assert_status_ok();
        rep.assert_text(DOC_MARKDOWN);
        // update
        app.put(&path)
            .text(DOC_MARKDOWN_UPDATED)
            .await
            .assert_status_ok();
        // check update
        app.get(&path).await.assert_text(DOC_MARKDOWN_UPDATED);
        // check index
        let rep = app.get("/index").await;
        rep.assert_status_ok();
        rep.assert_text(format!("\n[{id}](post/{id})\n"));
        // check rss
        let rep = app.get("/rss").await;
        rep.assert_status_ok();
        rep.assert_text(format!("<?xml version=\"1.0\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\"><title>My blog</title><id></id><updated>1970-01-01T00:00:00+00:00</updated><link href=\"https://blog.example.net/rss\" rel=\"alternate\"/><subtitle>A collection of my thoughts</subtitle><entry><title>{id}</title><id></id><updated>1970-01-01T00:00:00+00:00</updated><link href=\"https://blog.example.net/post/{id}\" rel=\"alternate\"/><content>{}</content></entry></feed>", DOC_MARKDOWN_UPDATED));
        // delete
        app.delete(&path).await.assert_status_ok();
        // check deletion.
        app.get(&path).await.assert_status_not_found();
        // check index
        let rep = app.get("/index").await;
        rep.assert_status_ok();
        rep.assert_text("");
        // check rss
        let rep = app.get("/rss").await;
        rep.assert_status_ok();
        rep.assert_text("<?xml version=\"1.0\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\"><title>My blog</title><id></id><updated>1970-01-01T00:00:00+00:00</updated><link href=\"https://blog.example.net/rss\" rel=\"alternate\"/><subtitle>A collection of my thoughts</subtitle></feed>");
        Ok(())
    }
}
