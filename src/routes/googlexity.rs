use actix_web::{web::Json, HttpResponse, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::error::Error;

use crate::{
    constants::utility::log_error,
    models::google_models::{SearchRequest, SearchResponse, SearchResult},
};

pub async fn google_search(query: &str) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    let search_api_key = std::env::var("SEARCH_API_KEY").unwrap();
    let search_engine_id = std::env::var("SEARCH_ENGINE_ID").unwrap();

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

    if !google_search_response.status().is_success() {
        return Err(format!("HTTP error! status: {}", google_search_response.status()).into());
    }

    let google_search_response_json: SearchResponse =
        google_search_response.json::<SearchResponse>().await?;

    let items = google_search_response_json.items;

    Ok(items)
}

pub async fn search(body: Json<SearchRequest>) -> Result<HttpResponse, Box<dyn Error>> {
    let search_items = google_search(&body.query).await;

    match search_items {
        Ok(search_items) => Ok(HttpResponse::Ok().json(search_items)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}
