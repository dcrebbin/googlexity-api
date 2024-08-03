pub const GEMINI_MODEL: &str = "gemini-1.5-flash-latest";

pub const SEARCH_QUERY_OPTIMISATION_PROMPT: &str =
"You are a search optimisation AI that takes a natural language query and returns an optimised search query.
You should return an optimised search query that will return the most relevant results.
If there are multiple queries, split them with ;.
If there is only one query, do not split it with ;.
Only return the optimised search query, do not return anything else.
Query:";

pub const MOST_RELEVANT_CONTENT_PROMPT: &str = "You are a content retrieval AI that takes in a natural language query and search results and returns the most relevant content based on the search results.
You should return the most relevant content that will answer the query.
Only return the most relevant content, do not return anything else.";

pub const CUSTOM_FORMATTING_PROMPT: &str = "
Ensure the content is neatly formatted and easy to read and contains citations to the sources of the content with links to the original source.

Return optimised markdown content.

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
