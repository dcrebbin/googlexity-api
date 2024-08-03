pub const GEMINI_MODEL: &str = "gemini-1.5-flash-latest";

pub const SEARCH_QUERY_OPTIMISATION_PROMPT: &str =
"You are a search optimisation AI that takes a natural language query and returns an optimised search query.
You should return an optimised search query that will return the most relevant results.
If there are multiple queries, split them with ;.
Only return the optimised search query, do not return anything else.
Query:";

pub const MOST_RELEVANT_CONTENT_PROMPT: &str =
"You are a content retrieval AI that takes in a natural language query and search results and returns the most relevant content based on the search results.
You should return the most relevant content that will answer the query.
Only return the most relevant content, do not return anything else.

Ensure the content is neatly formatted and easy to read and contains citations to the sources of the content with links to the original source.

Return optimised markdown content.

Query:";
