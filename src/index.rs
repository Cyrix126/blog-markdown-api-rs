use chrono::{format::StrftimeItems, Locale, NaiveDateTime};

use crate::{error::AppError, utils::detect_title_md, AppState};
const FORMAT_FILENAME: &str = "%Y-%m-%d %H:%M:%S";
const FORMAT_DATETIME: &str = "";

pub fn generate_index(state: &AppState, locale: Locale) -> Result<String, AppError> {
    let mut markdown = String::new();
    let mut paths: Vec<_> = std::fs::read_dir(&state.config.assets_path)?
        .map(|r| r.unwrap())
        .collect();
    paths.sort_unstable_by_key(|dir| dir.file_name());
    paths.reverse();
    for file in paths {
        let filename = file
            .file_name()
            .into_string()
            .map_err(AppError::OsString)?
            .replace(".md", "");
        let link = format!("(post/{filename})");
        if let Ok(date) = NaiveDateTime::parse_from_str(&filename, FORMAT_FILENAME) {
            let fmtr =
                date.format_with_items(StrftimeItems::new_with_locale(FORMAT_DATETIME, locale));
            let public_date = fmtr.to_string();
            let line;
            if let Some(title) = detect_title_md(&file.path())? {
                line = format!("\n**{}**  \n[{}]{}\n", title.trim(), public_date, link);
            } else {
                line = format!("\n[{}]{}\n", public_date, link);
            }
            markdown.push_str(&line);
        }
    }
    Ok(markdown)
}
