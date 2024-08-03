use crate::services::web_scraping::WebScraping;
use actix_web::{web::Json, HttpResponse, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::json;
use std::{error::Error, time::Instant};

use crate::constants::config::{GEMINI_MODEL_FLASH, GEMINI_MODEL_PRO};
use crate::{
    constants::{
        config::{
            CUSTOM_FORMATTING_PROMPT, MOST_RELEVANT_CONTENT_PROMPT,
            SEARCH_QUERY_OPTIMISATION_PROMPT,
        },
        utility::{log_error, log_query},
    },
    models::google_ai_models::{AiCompletionRequest, GoogleAiGenerateContentResponse},
    models::google_search_models::{SearchRequest, SearchResponse, SearchResult},
};

pub async fn search(body: Json<SearchRequest>) -> Result<HttpResponse, Box<dyn Error>> {
    let start_time = Instant::now();

    let query = body.query.clone();

    let optimised_search_response = if body.optimize_query.unwrap_or(true) {
        google_ai_completion(actix_web::web::Json(AiCompletionRequest {
            query: SEARCH_QUERY_OPTIMISATION_PROMPT.to_string() + &query,
            model: Some(GEMINI_MODEL_FLASH.to_string()),
        }))
        .await?
    } else {
        query
    };

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

    if let Some(max_results) = body.max_results {
        if let Ok(max) = usize::try_from(max_results) {
            search_results = search_results.into_iter().take(max).collect();
        }
    }

    let search_results_text = serde_json::to_string(&search_results)?;
    println!(
        "Initial search results content length: {}",
        search_results_text.len()
    );

    let updated_search_results = if body.depthfull_search.unwrap_or(false) {
        WebScraping::retrieve_all_website_text_content(search_results).await
    } else {
        search_results
    };

    let stringified_search_results = serde_json::to_string(&updated_search_results)?;
    if body.depthfull_search.unwrap_or(false) {
        let updated_search_results_length = stringified_search_results.len();
        println!(
            "Updated search results content length: {}",
            updated_search_results_length
        );
    }

    let ai_request = AiCompletionRequest {
        query: MOST_RELEVANT_CONTENT_PROMPT.to_string()
            + &body
                .custom_instructions
                .clone()
                .unwrap_or(CUSTOM_FORMATTING_PROMPT.to_string())
            + "\n\nQuery:\n"
            + &body.query.clone()
            + "\n\nSearch Results:\n"
            + &stringified_search_results,
        model: Some(body.model.clone().unwrap_or(GEMINI_MODEL_PRO.to_string())),
    };

    let ai_request_length = ai_request.query.len();
    println!("AI request length: {}", ai_request_length);

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
