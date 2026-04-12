//! ErukaClient — typed HTTP client for the Eruka context memory API.

use openeruka::{ErukaField, ErukaFieldWrite, ErukaEntity, ErukaEdge};
use crate::error::ClientError;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;

/// Typed async HTTP client for a hosted Eruka instance.
///
/// Authenticate with a service key (`sk:...`) or a JWT bearer token.
/// Get your key by signing up at [eruka.dirmacs.com](https://eruka.dirmacs.com).
pub struct ErukaClient {
    base_url: String,
    client: reqwest::Client,
}

impl ErukaClient {
    /// Create a new client with a service key (`X-Service-Key` auth).
    pub fn new(base_url: impl Into<String>, service_key: impl AsRef<str>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Service-Key",
            HeaderValue::from_str(service_key.as_ref()).expect("valid service key header"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("reqwest client build");

        Self { base_url: base_url.into(), client }
    }

    /// Create a new client with a JWT bearer token.
    pub fn new_with_jwt(base_url: impl Into<String>, token: impl AsRef<str>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token.as_ref()))
                .expect("valid bearer token"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("reqwest client build");

        Self { base_url: base_url.into(), client }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn check_status(resp: reqwest::Response) -> Result<reqwest::Response, ClientError> {
        let status = resp.status();
        if status.is_success() { return Ok(resp); }
        match status.as_u16() {
            401 => Err(ClientError::Unauthorized),
            404 => Err(ClientError::NotFound),
            409 => Err(ClientError::Conflict),
            code => {
                let body = resp.text().await.unwrap_or_default();
                Err(ClientError::Server { status: code, body })
            }
        }
    }

    /// Check if the Eruka instance is healthy.
    pub async fn health(&self) -> Result<bool, ClientError> {
        let resp = self.client.get(self.url("/health")).send().await?;
        Ok(resp.status().is_success())
    }

    /// Get a single field by path.
    pub async fn get_field(
        &self,
        workspace_id: &str,
        path: &str,
    ) -> Result<Option<ErukaField>, ClientError> {
        let url = format!("{}?workspace_id={}&path={}", self.url("/api/v1/context"), workspace_id, path);
        let resp = Self::check_status(self.client.get(&url).send().await?).await?;
        let body: Value = resp.json().await?;
        let fields = body.get("fields")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if fields.is_empty() {
            return Ok(None);
        }
        let field: ErukaField = serde_json::from_value(fields[0].clone())?;
        Ok(Some(field))
    }

    /// Get all fields matching a path prefix.
    pub async fn get_prefix(
        &self,
        workspace_id: &str,
        prefix: &str,
    ) -> Result<Vec<ErukaField>, ClientError> {
        let path = if prefix.ends_with('*') { prefix.to_string() } else { format!("{}/*", prefix) };
        let url = format!("{}?workspace_id={}&path={}", self.url("/api/v1/context"), workspace_id, path);
        let resp = Self::check_status(self.client.get(&url).send().await?).await?;
        let body: Value = resp.json().await?;
        let fields = body.get("fields")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let parsed: Vec<ErukaField> = fields.into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        Ok(parsed)
    }

    /// Write a context field.
    ///
    /// Returns `Err(ClientError::Conflict)` if a `Confirmed` field is written
    /// with a lower knowledge state — the server enforces the knowledge state invariant.
    pub async fn write_field(
        &self,
        workspace_id: &str,
        req: &ErukaFieldWrite,
    ) -> Result<ErukaField, ClientError> {
        let mut body = serde_json::to_value(req)?;
        body["workspace_id"] = serde_json::json!(workspace_id);
        let resp = Self::check_status(
            self.client.post(self.url("/api/v1/context"))
                .json(&body)
                .send()
                .await?
        ).await?;
        let field: ErukaField = resp.json().await?;
        Ok(field)
    }

    /// Get entities in the knowledge graph for a workspace.
    pub async fn get_entities(&self, workspace_id: &str) -> Result<Vec<ErukaEntity>, ClientError> {
        let url = format!("{}?workspace_id={}", self.url("/api/v1/entities"), workspace_id);
        let resp = Self::check_status(self.client.get(&url).send().await?).await?;
        let body: Value = resp.json().await?;
        let entities: Vec<ErukaEntity> = body.get("entities")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect())
            .unwrap_or_default();
        Ok(entities)
    }

    /// Get edges in the knowledge graph for a workspace.
    pub async fn get_edges(&self, workspace_id: &str) -> Result<Vec<ErukaEdge>, ClientError> {
        let url = format!("{}?workspace_id={}", self.url("/api/v1/edges"), workspace_id);
        let resp = Self::check_status(self.client.get(&url).send().await?).await?;
        let body: Value = resp.json().await?;
        let edges: Vec<ErukaEdge> = body.get("edges")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect())
            .unwrap_or_default();
        Ok(edges)
    }
}
