use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub model: Option<String>,
    pub max_results: Option<i32>,
    pub max_optimizations: Option<i32>,
    pub depthfull_search: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub kind: String,
    pub url: Url,
    pub queries: Queries,
    pub context: Context,
    #[serde(rename = "searchInformation")]
    pub search_information: SearchInformation,
    pub items: Vec<SearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Url {
    #[serde(rename = "type")]
    pub url_type: String,
    pub template: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Queries {
    pub request: Vec<QueryInfo>,
    #[serde(rename = "nextPage")]
    pub next_page: Vec<QueryInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryInfo {
    pub title: String,
    #[serde(rename = "totalResults")]
    pub total_results: String,
    #[serde(rename = "searchTerms")]
    pub search_terms: String,
    pub count: i32,
    #[serde(rename = "startIndex")]
    pub start_index: i32,
    #[serde(rename = "inputEncoding")]
    pub input_encoding: String,
    #[serde(rename = "outputEncoding")]
    pub output_encoding: String,
    pub safe: String,
    pub cx: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Context {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchInformation {
    #[serde(rename = "searchTime")]
    pub search_time: f64,
    #[serde(rename = "formattedSearchTime")]
    pub formatted_search_time: String,
    #[serde(rename = "totalResults")]
    pub total_results: String,
    #[serde(rename = "formattedTotalResults")]
    pub formatted_total_results: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_text_content: Option<String>,
    pub title: String,
    #[serde(rename = "htmlTitle")]
    pub html_title: String,
    pub link: String,
    #[serde(rename = "displayLink")]
    pub display_link: String,
    pub snippet: String,
    #[serde(rename = "htmlSnippet")]
    pub html_snippet: String,
    #[serde(rename = "formattedUrl")]
    pub formatted_url: String,
    #[serde(rename = "htmlFormattedUrl")]
    pub html_formatted_url: String,
    pub pagemap: Option<PageMap>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageMap {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hcard: Option<Vec<HCard>>,
    #[serde(rename = "cse_thumbnail", skip_serializing_if = "Option::is_none")]
    pub cse_thumbnail: Option<Vec<CseThumbnail>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metatags: Option<Vec<Metatags>>,
    #[serde(rename = "cse_image", skip_serializing_if = "Option::is_none")]
    pub cse_image: Option<Vec<CseImage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person: Option<Vec<Person>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HCard {
    #[serde(rename = "fn")]
    pub fn_: String,
    pub url: Option<String>,
    pub nickname: Option<String>,
    pub category: Option<String>,
    pub url_text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CseThumbnail {
    pub src: String,
    pub width: String,
    pub height: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metatags {
    #[serde(flatten)]
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CseImage {
    pub src: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub url: String,
}

#[derive(Deserialize)]
pub struct AiCompletionRequest {
    pub model: Option<String>,
    pub query: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleAiGenerateContentResponse {
    pub candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: UsageMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Candidate {
    pub content: Content,
    #[serde(rename = "finishReason")]
    pub finish_reason: String,
    pub index: u32,
    #[serde(rename = "safetyRatings")]
    pub safety_ratings: Vec<SafetyRating>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub parts: Vec<Part>,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Part {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: u32,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: u32,
}
