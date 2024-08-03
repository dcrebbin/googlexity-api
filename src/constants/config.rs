pub const GEMINI_MODEL_FLASH: &str = "gemini-1.5-flash-latest";
pub const GEMINI_MODEL_PRO: &str = "gemini-1.5-pro-latest";
pub const GEMINI_MODEL_EXPERIMENTAL: &str = "gemini-1.5-pro-exp-0801";

pub const SEARCH_QUERY_OPTIMISATION_PROMPT: &str =
"You are a search optimisation AI that takes a natural language query and returns an optimised search query.
You should return an optimised search query that will return the most relevant results.
If there are multiple queries, split them with ;.
If there is only one query, do not split it with ;.
Only return the optimised search query, do not return anything else.
Query:";

pub const MOST_RELEVANT_CONTENT_PROMPT: &str = "You are a content retrieval AI that takes in a natural language query and search results and returns the most relevant content based on the search results.
You should return the most relevant content that will answer the query.
ONLY USE factual information and DO NOT make up information.
Use real sources to support the information.
Obey the laws of physics, mathematics, and the laws of the universe. (real world distances, angles, etc.)
Think carefully before providing any information.
Take into consideration the liklihood, credibility, and reliability of the information.
Only return the most relevant content, do not return anything else.";

pub const CUSTOM_FORMATTING_PROMPT: &str = "
Return optimised markdown content with the following template:

Standard Template:

{information}

1.  **{result_1}**
    - {result_1_information_1}
    - {result_1_information_2}
    - {result_1_information_3}
    - Source: [{result_1_source}]({result_1_source})

2.  **{result_2}**
    - {result_2_information_1}
    - {result_2_information_2}
    - Source: [{result_2_source}]({result_2_source})

3.  **{result_3}**
    - {result_3_information_1}
    - {result_3_information_2}
    - Source: [{result_3_source}]({result_3_source})

{ending_message}

Financial Template:

{information}

Sources:
    - {source_1}
    - {source_2}
    - {source_3}

{markdown_table}

{ending_message}

Query:";

pub const DISALLOWED_URLS: &[&str] = &[
    "https://www.reddit.com",
    "https://x.com",
    "https://www.twitter.com",
];
