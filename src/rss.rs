use atom_syndication::{Content, Entry, Feed, Link, Text};

use crate::{error::AppError, utils::detect_title_md, AppState};
// Function to generate an Atom feed from Markdown files in a directory
pub fn generate_feed(state: &AppState) -> Result<String, AppError> {
    // Create a new Feed instance

    // Set feed title, subtitle, and link
    let mut link = Link::default();
    link.set_href(&format!("https://{}/rss", &state.config.hostname));
    let mut feed = Feed {
        title: state.config.rss_title.clone().into(),
        subtitle: Some(Text::from(
            state.config.rss_subtitle.clone().unwrap_or_default(),
        )),
        links: vec![link],
        ..Default::default()
    };

    // Iterate over the Markdown files in the directory
    for entry in std::fs::read_dir(&state.config.assets_path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() && path.extension().unwrap_or_default() == "md" {
            // Create a new Entry for the Markdown file
            let mut entry = Entry::default();

            // Set entry title, content, and link
            let file_name = path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .replace(".md", "");
            entry.title = detect_title_md(&path)?.unwrap_or(file_name.clone()).into();
            let mut content = Content::default();
            content.set_value(std::fs::read_to_string(path).unwrap());
            entry.content = Some(content);
            let mut link = Link::default();
            link.set_href(&format!(
                "https://{}/post/{file_name}",
                &state.config.hostname
            ));
            entry.links = vec![link];

            // Add the entry to the feed
            feed.entries.push(entry);
        }
    }

    Ok(feed.to_string())
}
