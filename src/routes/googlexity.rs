use actix_web::{web::Json, HttpResponse, Result};
use futures_util::stream::{self, StreamExt};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::json;
use std::sync::Arc;
use std::{error::Error, fs, path::Path, time::Instant};
use url::Url;

use crate::constants::config::{DISALLOWED_URLS, GEMINI_MODEL_FLASH, GEMINI_MODEL_PRO};
use crate::{
    constants::{
        config::{
            CUSTOM_FORMATTING_PROMPT, MOST_RELEVANT_CONTENT_PROMPT,
            SEARCH_QUERY_OPTIMISATION_PROMPT,
        },
        utility::{log_error, log_query},
    },
    models::google_models::{
        AiCompletionRequest, GoogleAiGenerateContentResponse, SearchRequest, SearchResponse,
        SearchResult,
    },
};

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
                actix_web::error::ErrorInternalServerError(format!("Failed to read file: {}", e))
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
    let mock_search_results = get_mock_search_results();
    let search_results = match mock_search_results {
        Ok(results) => results,
        Err(e) => return Err(e),
    };
    let updated_search_items = retrieve_all_website_text_content(search_results).await;

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

    let cleaned_text = clean_text(&text_content);

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

                let normalized_url = normalize_url(&cloned_link);
                println!("Normalized URL: {}", normalized_url);

                if DISALLOWED_URLS.contains(&normalized_url.as_str()) {
                    println!("URL {} is disallowed, skipping", normalized_url);
                    return item;
                }
                match scrape_website(&cloned_link, &client).await {
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

pub async fn search(body: Json<SearchRequest>) -> Result<HttpResponse, Box<dyn Error>> {
    let start_time = Instant::now();

    let optimised_search_response =
        google_ai_completion(actix_web::web::Json(AiCompletionRequest {
            query: SEARCH_QUERY_OPTIMISATION_PROMPT.to_string() + &body.query.clone(),
            model: Some(GEMINI_MODEL_FLASH.to_string()),
        }))
        .await?;

    log_query(&format!(
        "Optimised search response: {}",
        optimised_search_response
    ));

    let mut split_search_queries: Vec<String> = if optimised_search_response.contains(";") {
        optimised_search_response
            .split(';')
            .map(|s| s.replace("\n", ""))
            .collect()
    } else {
        vec![optimised_search_response]
    };

    if let Some(max_optimizations) = body.max_optimizations {
        if let Ok(max) = usize::try_from(max_optimizations) {
            split_search_queries = split_search_queries.into_iter().take(max).collect();
        }
    }

    log_query(&format!("Split search queries: {:?}", split_search_queries));

    let mut search_results: Vec<SearchResult> = Vec::new();

    for query in split_search_queries {
        let search_items = google_search(&query).await?;

        search_results.extend(search_items);
    }

    // cut results down to max_results
    if let Some(max_results) = body.max_results {
        if let Ok(max) = usize::try_from(max_results) {
            search_results = search_results.into_iter().take(max).collect();
        }
    }

    let updated_search_results = if body.depthfull_search.unwrap_or(false) {
        retrieve_all_website_text_content(search_results).await
    } else {
        search_results
    };

    let stringified_search_results = serde_json::to_string(&updated_search_results)?;

    let ai_request = AiCompletionRequest {
        query: MOST_RELEVANT_CONTENT_PROMPT.to_string()
            + CUSTOM_FORMATTING_PROMPT
            + "\n\nQuery:\n"
            + &body.query.clone()
            + "\n\nSearch Results:\n"
            + &stringified_search_results,
        model: Some(body.model.clone().unwrap_or(GEMINI_MODEL_PRO.to_string())),
    };

    let most_relevant_search_results =
        google_ai_completion(actix_web::web::Json(ai_request)).await?;

    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);
    println!("Googlexity Search time taken: {:?}", duration);

    Ok(HttpResponse::Ok().body(most_relevant_search_results))
}

pub async fn google_search(query: &str) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    let search_api_key = std::env::var("SEARCH_API_KEY").unwrap();
    let search_engine_id = std::env::var("SEARCH_ENGINE_ID").unwrap();

    let start_time = Instant::now();

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type".parse::<HeaderName>().unwrap(),
        "application/json".parse::<HeaderValue>().unwrap(),
    );
    let google_search_response = match client
        .get(format!(
            "https://www.googleapis.com/customsearch/v1?key={}&cx={}&q={}",
            search_api_key, search_engine_id, query
        ))
        .headers(headers)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            log_error(&format!("Request failed: {}", e));
            return Err(format!("Request failed: {}", e).into());
        }
    };

    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);
    println!("Google Search time taken: {:?}", duration);

    if !google_search_response.status().is_success() {
        return Err(format!("HTTP error! status: {}", google_search_response.status()).into());
    }

    let google_search_response_json: SearchResponse =
        google_search_response.json::<SearchResponse>().await?;

    let items = google_search_response_json.items;

    Ok(items)
}

pub async fn google_ai_completion(
    body: Json<AiCompletionRequest>,
) -> Result<String, Box<dyn Error>> {
    let gemini_api_key = std::env::var("GEMINI_API_KEY").unwrap();
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    let start_time = Instant::now();
    headers.insert(
        "Content-Type".parse::<HeaderName>().unwrap(),
        "application/json".parse::<HeaderValue>().unwrap(),
    );

    let function = "generateContent";
    let model = body.model.clone().unwrap_or(GEMINI_MODEL_FLASH.to_string());

    let google_ai_completion_response = match client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:{}?key={}",
            model, function, gemini_api_key
        ))
        .body(
            serde_json::to_string(&json!({
               "contents":[
                {
                    "parts":[
                        {
                            "text": body.query
                        }
                    ]
                }
               ]
            }))
            .unwrap(),
        )
        .headers(headers)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            log_error(&format!("Request failed: {}", e));
            return Ok(format!("Request failed: {}", e));
        }
    };

    if !google_ai_completion_response.status().is_success() {
        return Ok(format!(
            "HTTP error! status: {}",
            google_ai_completion_response.status()
        ));
    }

    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);
    println!(
        "Google AI Completion using {} time taken: {:?}",
        model, duration
    );

    let google_ai_completion_response_json: GoogleAiGenerateContentResponse =
        match google_ai_completion_response
            .json::<GoogleAiGenerateContentResponse>()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                log_error(&format!("Failed to parse JSON response: {}", e));
                return Ok(format!("Failed to parse JSON response: {}", e));
            }
        };

    let content = &google_ai_completion_response_json.candidates[0]
        .content
        .parts[0]
        .text;

    Ok(content.to_string())
}
