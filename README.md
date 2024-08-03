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


