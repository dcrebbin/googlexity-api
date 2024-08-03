# Googlexity API

![Googlexity](/public/googlexity.png)

**Googlexity** is a Rust Actix Web API that attempts to mimic [Perplexity.ai](https://perplexity.ai)'s capabilities.

It does this by:

- Optimizing search queries

- Searching Google via Google's Custom Search JSON API

- Scraping the text content of each search result item

- Using that context to generate a response using a custom markdown template that has minimal hallucinations

### Examples

**What is the stock price of Meta from the last 14 days?**

![Example 1](/public/example-1.png)

**What is the closest capybara from Melbourne?**

![Example 2](/public/example-2.png)

## Pros

- It's quite accurate

## Issues

- It's not very fast at the moment (But I have some ideas on how to speed it up)

## Setup 

1) Install Rust

2) Create a `.env` file based on `env.example`

3)
    - **local:** Run `cargo run`

    - **docker:** Run `docker compose up --build`

    - **deploy:** https://fly.io or https://console.cloud.google.com

## Usage

### Search

Endpoint: `http://127.0.0.1:8080/api/search`

Type: `POST`

Headers:

- `Content-Type: application/json`
- `x-api-key: <YOUR_API_KEY>` (set in .env)

Body:

```json
{
    "query": "",
     // (Required)
     // The query to search for
    "model": "",
     // (Optional: defaults to "gemini-1.5-pro-latest")
     // The Google AI model to use, see config.rs or 
     // https://ai.google.dev/gemini-api/docs/models/gemini for options
    "max_results": "", 
     // (Optional: defaults to infinite)
     // The maximum number of search results to use in the
     // context of the last AI query
    "optimize_query": "", 
     // (Optional: defaults to true)
     // Whether to optimize the query for search engines. 
    "max_optimizations": "", 
     // (Optional: defaults to infinite)
     // The maximum number of optimized queries that will be used 
    "depthfull_search": ""
     // (Optional: defaults to false)
     // Whether to perform a depthful search. 
     // a depthful search will scrape each website link
     // and then use the content in the context of the last AI query 
}
```
