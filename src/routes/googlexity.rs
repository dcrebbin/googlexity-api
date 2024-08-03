use actix_web::{web::Json, HttpResponse, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::json;
use std::error::Error;

use crate::{
    constants::utility::log_error,
    models::google_models::{
        AiCompletionRequest, GoogleAiGenerateContentResponse, SearchRequest, SearchResponse,
        SearchResult,
    },
};

pub async fn search(body: Json<SearchRequest>) -> Result<HttpResponse, Box<dyn Error>> {
    let search_items = google_search(&body.query).await;

    match search_items {
        Ok(search_items) => Ok(HttpResponse::Ok().json(search_items)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

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

pub async fn google_ai_completion(
    body: Json<AiCompletionRequest>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let gemini_api_key = std::env::var("GEMINI_API_KEY").unwrap();
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type".parse::<HeaderName>().unwrap(),
        "application/json".parse::<HeaderValue>().unwrap(),
    );

    let function = "generateContent";
    let model = body
        .model
        .clone()
        .unwrap_or("gemini-1.5-flash-latest".to_string());

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
            return Ok(HttpResponse::InternalServerError().body(format!("Request failed: {}", e)));
        }
    };

    if !google_ai_completion_response.status().is_success() {
        return Ok(HttpResponse::InternalServerError().body(format!(
            "HTTP error! status: {}",
            google_ai_completion_response.status()
        )));
    }
    let google_ai_completion_response_json: GoogleAiGenerateContentResponse =
        match google_ai_completion_response
            .json::<GoogleAiGenerateContentResponse>()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                log_error(&format!("Failed to parse JSON response: {}", e));
                return Ok(HttpResponse::InternalServerError()
                    .body(format!("Failed to parse JSON response: {}", e)));
            }
        };

    let content = &google_ai_completion_response_json.candidates[0]
        .content
        .parts[0]
        .text;

    Ok(HttpResponse::Ok().body(content.to_string()))
}
