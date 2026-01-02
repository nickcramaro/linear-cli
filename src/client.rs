use crate::error::{Error, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.linear.app/graphql";

pub struct LinearClient {
    http: reqwest::Client,
}

#[derive(Serialize)]
struct GraphQLRequest<T: Serialize> {
    query: String,
    variables: T,
}

#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

impl LinearClient {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("LINEAR_API_KEY").map_err(|_| Error::MissingApiKey)?;

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&api_key).expect("invalid api key format"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("failed to build http client");

        Ok(Self { http })
    }

    pub async fn query<V, T>(&self, query: &str, variables: V) -> Result<T>
    where
        V: Serialize,
        T: for<'de> Deserialize<'de>,
    {
        let request = GraphQLRequest {
            query: query.to_string(),
            variables,
        };

        let response = self.http.post(API_URL).json(&request).send().await?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::Unauthorized);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(60);
            return Err(Error::RateLimited(retry_after));
        }

        let gql_response: GraphQLResponse<T> = response.json().await?;

        if let Some(errors) = gql_response.errors {
            let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
            return Err(Error::GraphQL(messages.join(", ")));
        }

        gql_response
            .data
            .ok_or_else(|| Error::GraphQL("no data in response".to_string()))
    }
}
