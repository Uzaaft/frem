use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::types::{Issue, User};

const LINEAR_API_URL: &str = "https://api.linear.app/graphql";

pub struct LinearClient {
    client: reqwest::blocking::Client,
    api_key: String,
}

#[derive(Debug, Serialize)]
struct GraphQLRequest {
    query: String,
    variables: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct IssuesResponse {
    issues: Connection<Issue>,
}

#[derive(Debug, Deserialize)]
struct Connection<T> {
    edges: Vec<Edge<T>>,
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
struct Edge<T> {
    node: T,
    cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

impl LinearClient {
    pub fn new(api_key: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&api_key)?);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { client, api_key })
    }
    
    pub fn new_with_oauth(access_token: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { client, api_key: access_token })
    }

    pub fn get_issues(&self, limit: i32) -> Result<Vec<Issue>> {
        let query = r#"
            query GetIssues($first: Int!) {
                issues(first: $first) {
                    edges {
                        node {
                            id
                            identifier
                            title
                            description
                            priority
                            createdAt
                            updatedAt
                            state {
                                id
                                name
                                color
                            }
                            assignee {
                                id
                                name
                                email
                            }
                        }
                    }
                }
            }
        "#;

        let variables = json!({
            "first": limit
        });

        let request = GraphQLRequest {
            query: query.to_string(),
            variables: Some(variables),
        };

        let response: GraphQLResponse<IssuesResponse> = self
            .client
            .post(LINEAR_API_URL)
            .json(&request)
            .send()?
            .json()?;

        if let Some(errors) = response.errors {
            let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("GraphQL errors: {}", error_messages.join(", "));
        }

        let issues = response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in response"))?
            .issues
            .edges
            .into_iter()
            .map(|edge| edge.node)
            .collect();

        Ok(issues)
    }

    pub fn get_viewer(&self) -> Result<User> {
        let query = r#"
            query GetViewer {
                viewer {
                    id
                    name
                    email
                }
            }
        "#;

        let request = GraphQLRequest {
            query: query.to_string(),
            variables: None,
        };

        #[derive(Deserialize)]
        struct ViewerResponse {
            viewer: User,
        }

        let response: GraphQLResponse<ViewerResponse> = self
            .client
            .post(LINEAR_API_URL)
            .json(&request)
            .send()?
            .json()?;

        if let Some(errors) = response.errors {
            let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("GraphQL errors: {}", error_messages.join(", "));
        }

        let viewer = response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in response"))?
            .viewer;

        Ok(viewer)
    }
}