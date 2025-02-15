use actix_web::{HttpResponse, Result};
use futures_util::stream::{self, StreamExt};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use scraper::{Html, Selector};
use std::sync::Arc;
use std::{error::Error, fs, path::Path, time::Instant};
use url::Url;

use crate::constants::config::DISALLOWED_URLS;
use crate::constants::utility::log_error;
use crate::models::google_search_models::{SearchResponse, SearchResult};

pub struct WebScraping;

impl WebScraping {
    fn normalize_url(url: &str) -> String {
        match Url::parse(url) {
            Ok(mut parsed_url) => {
                parsed_url.set_scheme("https").unwrap_or(());
                parsed_url.set_path("");
                parsed_url.set_query(None);
                parsed_url.set_fragment(None);
                parsed_url.to_string().trim_end_matches('/').to_string()
            }
            Err(_) => url.to_string(),
        }
    }

    pub async fn retrieve_all_website_text_content(body: Vec<SearchResult>) -> Vec<SearchResult> {
        let start_time = Instant::now();
        let client = Arc::new(reqwest::Client::new());

        let updated_search_results = stream::iter(body)
            .map(|mut item| {
                let client = Arc::clone(&client);
                async move {
                    let cloned_link = item.link.clone();

                    let normalized_url = Self::normalize_url(&cloned_link);
                    println!("Normalized URL: {}", normalized_url);

                    if DISALLOWED_URLS.contains(&normalized_url.as_str()) {
                        println!("URL {} is disallowed, skipping", normalized_url);
                        return item;
                    }
                    match Self::scrape_website(&cloned_link, &client).await {
                        Ok(content) => {
                            item.website_text_content = Some(content);
                        }
                        Err(e) => {
                            log_error(&format!(
                                "Failed to scrape website {}: {}",
                                normalized_url, e
                            ));
                        }
                    }
                    item
                }
            })
            .buffer_unordered(10) // Process up to 10 requests concurrently
            .collect::<Vec<_>>()
            .await;

        let end_time = Instant::now();
        let duration = end_time.duration_since(start_time);
        println!("Full scraping time taken: {:?}", duration);

        updated_search_results
    }

    pub fn get_mock_search_results() -> Result<Vec<SearchResult>, actix_web::Error> {
        let dir_path = "./src/constants/mock/google_search/test";

        if !Path::new(dir_path).exists() {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "Directory does not exist: {}",
                dir_path
            )));
        }
        let mut custom_models: Vec<SearchResponse> = Vec::new();

        let entries = fs::read_dir(dir_path).map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to read directory: {}", e))
        })?;

        for entry_result in entries {
            let entry = entry_result.map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Failed to read entry: {}", e))
            })?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let contents = fs::read_to_string(&path).map_err(|e| {
                    actix_web::error::ErrorInternalServerError(format!(
                        "Failed to read file: {}",
                        e
                    ))
                })?;
                match serde_json::from_str::<SearchResponse>(&contents) {
                    Ok(response) => custom_models.push(response),
                    Err(e) => println!("Error parsing {}: {}", path.display(), e),
                }
            }
        }

        let mut all_search_items: Vec<SearchResult> = Vec::new();

        for response in custom_models {
            all_search_items.extend(response.items);
        }

        Ok(all_search_items)
    }

    pub async fn retrieve_relevant_search_data_mock() -> Result<HttpResponse, actix_web::Error> {
        let mock_search_results = Self::get_mock_search_results();
        let search_results = match mock_search_results {
            Ok(results) => results,
            Err(e) => return Err(e),
        };
        let updated_search_items = Self::retrieve_all_website_text_content(search_results).await;

        Ok(HttpResponse::Ok().json(updated_search_items))
    }

    pub async fn scrape_website(
        url: &str,
        client: &Client,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let start_time = Instant::now();
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent".parse::<HeaderName>().unwrap(),
            "Googlexity/0.1.0".parse::<HeaderValue>().unwrap(),
        );
        headers.insert(
            "Accept".parse::<HeaderName>().unwrap(),
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8"
                .parse::<HeaderValue>()
                .unwrap(),
        );
        let response = client.get(url).headers(headers).send().await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);

        let allowed_tags = [
            "h1", "h2", "h3", "h4", "h5", "h6", "p", "span", "a", "article", "sup", "table", "img",
            "link", "figure",
        ];

        let mut text_content = String::new();

        for tag in &allowed_tags {
            let selector = Selector::parse(tag).unwrap();
            for element in document.select(&selector) {
                text_content.push_str(&element.text().collect::<Vec<_>>().join(" "));
                text_content.push(' ');
            }
        }

        let cleaned_text = Self::clean_text(&text_content);

        let end_time = Instant::now();
        let duration = end_time.duration_since(start_time);
        println!("Scraping time taken for {}: {:?}", url, duration);
        Ok(cleaned_text)
    }

    fn clean_text(input: &str) -> String {
        // Remove script tags and their content
        let script_regex = Regex::new(r"(?i)<script\b[^>]*>[\s\S]*?</script>").unwrap();
        let without_scripts = script_regex.replace_all(input, "");

        // Remove style tags and their content
        let style_regex = Regex::new(r"(?i)<style\b[^>]*>[\s\S]*?</style>").unwrap();
        let without_styles = style_regex.replace_all(&without_scripts, "");

        // Remove inline JavaScript events
        let js_events_regex = Regex::new(r#"(?i)\s(on\w+)="[^"]*""#).unwrap();
        let without_js_events = js_events_regex.replace_all(&without_styles, "");

        // Remove HTML comments
        let comments_regex = Regex::new(r"<!--[\s\S]*?-->").unwrap();
        let without_comments = comments_regex.replace_all(&without_js_events, "");

        // Remove remaining HTML tags
        let tags_regex = Regex::new(r"<[^>]+>").unwrap();
        let without_tags = tags_regex.replace_all(&without_comments, "");

        // Remove extra whitespace
        let whitespace_regex = Regex::new(r"\s+").unwrap();
        let cleaned_text = whitespace_regex
            .replace_all(&without_tags, " ")
            .trim()
            .to_string();

        cleaned_text
    }
}
