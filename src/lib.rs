//! Async Rust client for the Ignite Containers control plane.
//!
//! ```no_run
//! # async fn run() -> Result<(), ignite_containers_client::Error> {
//! let client = ignite_containers_client::Client::new("https://games.example.com", "ic_...")?;
//! let servers = client.list_servers(ignite_containers_client::ServerQuery::default()).await?;
//! println!("{} servers", servers.len());
//! # Ok(())
//! # }
//! ```

use chrono::{DateTime, Utc};
use ignite_containers_core::ids::{
    AllocationId, ApiKeyId, DatabaseHostId, NodeId, ServerId, TemplateId,
};
use ignite_containers_core::job::{Job, PowerSignal};
use ignite_containers_core::limits::{Limits, NodeCapacity};
use ignite_containers_core::models::{
    Allocation, ApiKey, AuditEvent, Backup, DatabaseHost, EmailMessage, Node, NodeDriver,
    PortProtocol, ProxyRoute, ProxyTls, Schedule, ScheduleAction, Server, ServerDatabase,
    ServerDetail, ServerState, SftpCredential,
};
use ignite_containers_core::template::{
    AutoUpdate, Channel, Template, TemplateRecord, TemplateVersionRecord,
};
use ignite_containers_proto::{FileContent, FileEntry, FileList, NodeState};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid base URL: {0}")]
    InvalidUrl(String),
    #[error("transport error: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("could not decode response: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("HTTP {status} ({code}): {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
        details: Option<Value>,
    },
}

impl Error {
    #[must_use]
    pub fn status(&self) -> Option<u16> {
        match self {
            Error::Api { status, .. } => Some(*status),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_not_found(&self) -> bool {
        self.status() == Some(404)
    }

    #[must_use]
    pub fn is_conflict(&self) -> bool {
        self.status() == Some(409)
    }
}

#[derive(Debug, Deserialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    code: String,
    message: String,
    #[serde(default)]
    details: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreatedKey {
    pub key: ApiKey,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct KeyList {
    pub keys: Vec<ApiKey>,
}

#[derive(Debug, Deserialize)]
pub struct NodeWithToken {
    #[serde(flatten)]
    pub node: Node,
    pub agent_token: String,
}

#[derive(Debug, Deserialize)]
pub struct NodeList {
    pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct RotatedToken {
    pub id: NodeId,
    pub agent_token: String,
}

#[derive(Debug, Deserialize)]
pub struct AllocationList {
    pub allocations: Vec<Allocation>,
}

#[derive(Debug, Deserialize)]
pub struct AllocationResult {
    pub created: usize,
    pub requested: usize,
    pub skipped: usize,
}

#[derive(Debug, Deserialize)]
pub struct TemplateList {
    pub templates: Vec<TemplateRecord>,
}

#[derive(Debug, Deserialize)]
pub struct ImportResult {
    pub saved: bool,
    pub template: Template,
    #[serde(default)]
    pub record: Option<TemplateRecord>,
}

#[derive(Debug, Deserialize)]
pub struct ServerList {
    pub servers: Vec<Server>,
}

#[derive(Debug, Deserialize)]
pub struct JobList {
    pub jobs: Vec<Job>,
}

#[derive(Debug, Deserialize)]
pub struct Me {
    pub key_id: ApiKeyId,
    pub name: String,
    pub scopes: Vec<String>,
    pub server_ids: Vec<ServerId>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseHostList {
    pub hosts: Vec<DatabaseHost>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseList {
    pub databases: Vec<ServerDatabase>,
}

#[derive(Debug, Deserialize)]
pub struct CreatedDatabase {
    #[serde(flatten)]
    pub database: ServerDatabase,
    pub password: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseHostTest {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct BackupList {
    pub backups: Vec<Backup>,
}

#[derive(Debug, Deserialize)]
pub struct BulkResult {
    pub server_id: ServerId,
    pub success: bool,
    pub error: Option<String>,
    pub job_id: Option<ignite_containers_core::ids::JobId>,
}

#[derive(Debug, Deserialize)]
pub struct SftpList {
    pub credentials: Vec<SftpCredential>,
}

#[derive(Debug, Deserialize)]
pub struct CreatedSftp {
    #[serde(flatten)]
    pub credential: SftpCredential,
    pub host: String,
    pub port: u16,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailList {
    pub emails: Vec<EmailMessage>,
}

#[derive(Debug, Deserialize)]
pub struct BulkResponse {
    pub results: Vec<BulkResult>,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleList {
    pub schedules: Vec<Schedule>,
}

#[derive(Debug, Deserialize)]
pub struct ServerEnvironment {
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct AuditList {
    pub events: Vec<AuditEvent>,
}

#[derive(Debug, Deserialize)]
pub struct TemplateVersionList {
    pub versions: Vec<TemplateVersionRecord>,
}

#[derive(Debug, Serialize)]
pub struct PublishTemplateVersion {
    pub template: Template,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<Channel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyRouteList {
    pub routes: Vec<ProxyRoute>,
}

#[derive(Debug, Serialize)]
pub struct CreateProxyRoute {
    pub domain: String,
    pub upstream: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<ProxyTls>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UpdateProxyRoute {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<ProxyTls>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyMutation {
    pub route: ProxyRoute,
    pub applied: bool,
    #[serde(default)]
    pub warning: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyDeleteResult {
    pub deleted: bool,
    pub applied: bool,
    #[serde(default)]
    pub warning: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyStatus {
    pub configured: bool,
    #[serde(default)]
    pub admin_url: Option<String>,
    pub reachable: bool,
    pub routes: usize,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyReconcileResult {
    pub configured: bool,
    pub applied: bool,
    pub routes: usize,
    #[serde(default)]
    pub warning: Option<String>,
    #[serde(default)]
    pub caddy: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct FeedResult {
    pub published: Vec<FeedPublished>,
    pub skipped: Vec<FeedSkipped>,
}

#[derive(Debug, Deserialize)]
pub struct FeedPublished {
    pub slug: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct FeedSkipped {
    pub slug: String,
    pub reason: String,
}

// ------------------------------------------------------------------ requests

#[derive(Debug, Serialize)]
pub struct CreateNode {
    pub name: String,
    pub fqdn: String,
    pub endpoint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub driver: Option<NodeDriver>,
    #[serde(default)]
    pub maintenance: bool,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity: Option<NodeCapacity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_token: Option<String>,
}

impl CreateNode {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        fqdn: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            fqdn: fqdn.into(),
            endpoint: endpoint.into(),
            driver: Some(NodeDriver::Docker),
            maintenance: false,
            labels: BTreeMap::new(),
            capacity: None,
            agent_token: None,
        }
    }
}

#[derive(Debug, Serialize, Default)]
pub struct UpdateNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fqdn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<NodeDriver>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintenance: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct CreateAllocations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<PortProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl CreateServer {
    #[must_use]
    pub fn new(node_id: NodeId, template_id: TemplateId, name: impl Into<String>) -> Self {
        Self {
            node_id,
            template_id,
            template_version: None,
            name: name.into(),
            description: String::new(),
            external_ref: None,
            idempotency_key: None,
            labels: BTreeMap::new(),
            image: None,
            variables: BTreeMap::new(),
            limits: None,
            allocations: Vec::new(),
            auto_allocations: Some(1),
            protocol: None,
        }
    }
}

impl CreateAllocations {
    #[must_use]
    pub fn range(spec: impl Into<String>) -> Self {
        Self {
            ip: Some("0.0.0.0".into()),
            ports: Some(spec.into()),
            start: None,
            end: None,
            protocol: Some(PortProtocol::Tcp),
            label: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ImportEgg {
    pub egg: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<Channel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    pub save: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
}

impl ImportEgg {
    #[must_use]
    pub fn new(egg: Value) -> Self {
        Self {
            egg,
            version: None,
            channel: None,
            slug: None,
            save: true,
            signature: None,
            public_key: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateServer {
    pub node_id: NodeId,
    pub template_id: TemplateId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_version: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<Limits>,
    #[serde(default)]
    pub allocations: Vec<AllocationId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_allocations: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<PortProtocol>,
}

#[derive(Debug, Serialize, Default)]
pub struct UpdateServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<Limits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_version: Option<String>,
}

#[derive(Debug, Default)]
pub struct ServerQuery {
    pub node_id: Option<NodeId>,
    pub state: Option<ServerState>,
    pub external_ref: Option<String>,
    pub name: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CreateKey {
    pub name: String,
    pub scopes: Vec<String>,
    #[serde(default)]
    pub server_ids: Vec<ServerId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CreateBackup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateSchedule {
    pub name: String,
    pub cron: String,
    pub actions: Vec<ScheduleAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateSchedule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<ScheduleAction>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct WriteFile {
    pub path: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateDatabaseHost {
    pub name: String,
    pub host: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    pub admin_user: String,
    pub admin_password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_databases: Option<u32>,
}

#[derive(Debug, Default, Serialize)]
pub struct CreateServerDatabase {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_id: Option<ignite_containers_core::ids::DatabaseHostId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
}

// ------------------------------------------------------------------- client

#[derive(Clone)]
pub struct Client {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
    /// Separate client without a response timeout for long-lived SSE streams.
    stream_http: reqwest::Client,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

impl Client {
    pub fn new(base_url: impl AsRef<str>, api_key: impl Into<String>) -> Result<Self, Error> {
        let base_url = base_url.as_ref().trim_end_matches('/').to_string();
        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            return Err(Error::InvalidUrl(base_url));
        }
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(concat!(
                "ignite-containers-client-rust/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()?;
        let stream_http = reqwest::Client::builder()
            .user_agent(concat!(
                "ignite-containers-client-rust/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()?;
        Ok(Self {
            base_url,
            api_key: api_key.into(),
            http,
            stream_http,
        })
    }

    #[must_use]
    pub fn with_http_client(mut self, http: reqwest::Client) -> Self {
        self.stream_http = http.clone();
        self.http = http;
        self
    }

    async fn send<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<T, Error> {
        let mut request = self
            .http
            .request(method, format!("{}{}", self.base_url, path))
            .bearer_auth(&self.api_key)
            .header("accept", "application/json");
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(parse_error(status, &text));
        }
        Ok(serde_json::from_str(&text)?)
    }

    async fn send_empty(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<(), Error> {
        let mut request = self
            .http
            .request(method, format!("{}{}", self.base_url, path))
            .bearer_auth(&self.api_key);
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(parse_error(status, &text));
        }
        Ok(())
    }

    // auth -----------------------------------------------------------------

    pub async fn me(&self) -> Result<Me, Error> {
        self.send(Method::GET, "/api/v1/me", None).await
    }

    // api keys -------------------------------------------------------------

    pub async fn list_api_keys(&self) -> Result<Vec<ApiKey>, Error> {
        let list: KeyList = self.send(Method::GET, "/api/v1/api-keys", None).await?;
        Ok(list.keys)
    }

    pub async fn create_api_key(&self, request: &CreateKey) -> Result<CreatedKey, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/api-keys", Some(&body))
            .await
    }

    pub async fn revoke_api_key(&self, id: ApiKeyId) -> Result<(), Error> {
        self.send_empty(Method::DELETE, &format!("/api/v1/api-keys/{id}"), None)
            .await
    }

    // nodes ----------------------------------------------------------------

    pub async fn list_nodes(&self) -> Result<Vec<Node>, Error> {
        let list: NodeList = self.send(Method::GET, "/api/v1/nodes", None).await?;
        Ok(list.nodes)
    }

    pub async fn create_node(&self, request: &CreateNode) -> Result<NodeWithToken, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/nodes", Some(&body)).await
    }

    pub async fn get_node(&self, id: NodeId) -> Result<Node, Error> {
        self.send(Method::GET, &format!("/api/v1/nodes/{id}"), None)
            .await
    }

    pub async fn update_node(&self, id: NodeId, request: &UpdateNode) -> Result<Node, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::PATCH, &format!("/api/v1/nodes/{id}"), Some(&body))
            .await
    }

    pub async fn delete_node(&self, id: NodeId) -> Result<(), Error> {
        self.send_empty(Method::DELETE, &format!("/api/v1/nodes/{id}"), None)
            .await
    }

    pub async fn rotate_node_token(&self, id: NodeId) -> Result<RotatedToken, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/nodes/{id}/rotate-token"),
            Some(&body),
        )
        .await
    }

    // host updates ---------------------------------------------------------

    pub async fn node_updates(
        &self,
        id: NodeId,
    ) -> Result<ignite_containers_proto::HostUpdateStatus, Error> {
        self.send(Method::GET, &format!("/api/v1/nodes/{id}/updates"), None)
            .await
    }

    pub async fn check_node_updates(
        &self,
        id: NodeId,
    ) -> Result<ignite_containers_proto::HostUpdateStatus, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/nodes/{id}/updates/check"),
            Some(&body),
        )
        .await
    }

    pub async fn apply_node_updates(&self, id: NodeId, reboot: bool) -> Result<Value, Error> {
        let body = serde_json::json!({"reboot": reboot});
        self.send(
            Method::POST,
            &format!("/api/v1/nodes/{id}/updates/apply"),
            Some(&body),
        )
        .await
    }

    pub async fn reboot_node(&self, id: NodeId) -> Result<Value, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/nodes/{id}/reboot"),
            Some(&body),
        )
        .await
    }

    // databases ------------------------------------------------------------

    pub async fn list_database_hosts(&self) -> Result<Vec<DatabaseHost>, Error> {
        let list: DatabaseHostList = self
            .send(Method::GET, "/api/v1/database-hosts", None)
            .await?;
        Ok(list.hosts)
    }

    pub async fn create_database_host(
        &self,
        request: &CreateDatabaseHost,
    ) -> Result<DatabaseHost, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/database-hosts", Some(&body))
            .await
    }

    pub async fn delete_database_host(
        &self,
        id: ignite_containers_core::ids::DatabaseHostId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/database-hosts/{id}"),
            None,
        )
        .await
    }

    pub async fn test_database_host(
        &self,
        id: ignite_containers_core::ids::DatabaseHostId,
    ) -> Result<DatabaseHostTest, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/database-hosts/{id}/test"),
            Some(&body),
        )
        .await
    }

    pub async fn list_server_databases(
        &self,
        server_id: ServerId,
    ) -> Result<Vec<ServerDatabase>, Error> {
        let list: DatabaseList = self
            .send(
                Method::GET,
                &format!("/api/v1/servers/{server_id}/databases"),
                None,
            )
            .await?;
        Ok(list.databases)
    }

    pub async fn create_server_database(
        &self,
        server_id: ServerId,
        request: &CreateServerDatabase,
    ) -> Result<CreatedDatabase, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{server_id}/databases"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_server_database(
        &self,
        server_id: ServerId,
        database_id: ignite_containers_core::ids::ServerDatabaseId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/servers/{server_id}/databases/{database_id}"),
            None,
        )
        .await
    }

    // sftp -----------------------------------------------------------------

    pub async fn create_sftp_credential(
        &self,
        server_id: ServerId,
        ttl_seconds: Option<i64>,
    ) -> Result<CreatedSftp, Error> {
        let body = serde_json::json!({"ttl_seconds": ttl_seconds});
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{server_id}/sftp"),
            Some(&body),
        )
        .await
    }

    pub async fn list_sftp_credentials(
        &self,
        server_id: ServerId,
    ) -> Result<Vec<SftpCredential>, Error> {
        let list: SftpList = self
            .send(
                Method::GET,
                &format!("/api/v1/servers/{server_id}/sftp"),
                None,
            )
            .await?;
        Ok(list.credentials)
    }

    pub async fn revoke_sftp_credential(
        &self,
        server_id: ServerId,
        credential_id: ignite_containers_core::ids::SftpCredentialId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/servers/{server_id}/sftp/{credential_id}"),
            None,
        )
        .await
    }

    // notifications --------------------------------------------------------

    pub async fn send_email(
        &self,
        to: Vec<String>,
        subject: &str,
        body: &str,
    ) -> Result<EmailMessage, Error> {
        let payload = serde_json::json!({"to": to, "subject": subject, "body": body});
        self.send(Method::POST, "/api/v1/notifications/email", Some(&payload))
            .await
    }

    pub async fn list_emails(&self, limit: i64) -> Result<Vec<EmailMessage>, Error> {
        let list: EmailList = self
            .send(
                Method::GET,
                &format!("/api/v1/notifications/emails?limit={limit}"),
                None,
            )
            .await?;
        Ok(list.emails)
    }

    pub async fn retry_email(
        &self,
        id: ignite_containers_core::ids::EmailId,
    ) -> Result<EmailMessage, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/notifications/emails/{id}/retry"),
            Some(&body),
        )
        .await
    }

    pub async fn notification_config(&self) -> Result<Value, Error> {
        self.send(Method::GET, "/api/v1/notifications/config", None)
            .await
    }

    // bulk -----------------------------------------------------------------

    pub async fn bulk_power(
        &self,
        server_ids: Vec<ServerId>,
        signal: PowerSignal,
    ) -> Result<Vec<BulkResult>, Error> {
        let body = serde_json::json!({"server_ids": server_ids, "signal": signal});
        let response: BulkResponse = self
            .send(Method::POST, "/api/v1/servers/bulk/power", Some(&body))
            .await?;
        Ok(response.results)
    }

    pub async fn bulk_command(
        &self,
        server_ids: Vec<ServerId>,
        command: &str,
    ) -> Result<Vec<BulkResult>, Error> {
        let body = serde_json::json!({"server_ids": server_ids, "command": command});
        let response: BulkResponse = self
            .send(Method::POST, "/api/v1/servers/bulk/command", Some(&body))
            .await?;
        Ok(response.results)
    }

    // backups --------------------------------------------------------------

    pub async fn list_backups(&self, server_id: ServerId) -> Result<Vec<Backup>, Error> {
        let list: BackupList = self
            .send(
                Method::GET,
                &format!("/api/v1/servers/{server_id}/backups"),
                None,
            )
            .await?;
        Ok(list.backups)
    }

    pub async fn create_backup(
        &self,
        server_id: ServerId,
        request: &CreateBackup,
    ) -> Result<Backup, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{server_id}/backups"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_backup(
        &self,
        server_id: ServerId,
        backup_id: ignite_containers_core::ids::BackupId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/servers/{server_id}/backups/{backup_id}"),
            None,
        )
        .await
    }

    pub async fn restore_backup(
        &self,
        server_id: ServerId,
        backup_id: ignite_containers_core::ids::BackupId,
    ) -> Result<Backup, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{server_id}/backups/{backup_id}/restore"),
            Some(&body),
        )
        .await
    }

    pub async fn download_backup(
        &self,
        server_id: ServerId,
        backup_id: ignite_containers_core::ids::BackupId,
    ) -> Result<Vec<u8>, Error> {
        let response = self
            .http
            .get(format!(
                "{}/api/v1/servers/{server_id}/backups/{backup_id}/download",
                self.base_url
            ))
            .bearer_auth(&self.api_key)
            .timeout(std::time::Duration::from_secs(600))
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(parse_error(status, &text));
        }
        Ok(response.bytes().await?.to_vec())
    }

    // schedules ------------------------------------------------------------

    pub async fn list_schedules(&self, server_id: ServerId) -> Result<Vec<Schedule>, Error> {
        let list: ScheduleList = self
            .send(
                Method::GET,
                &format!("/api/v1/servers/{server_id}/schedules"),
                None,
            )
            .await?;
        Ok(list.schedules)
    }

    pub async fn create_schedule(
        &self,
        server_id: ServerId,
        request: &CreateSchedule,
    ) -> Result<Schedule, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{server_id}/schedules"),
            Some(&body),
        )
        .await
    }

    pub async fn update_schedule(
        &self,
        schedule_id: ignite_containers_core::ids::ScheduleId,
        request: &UpdateSchedule,
    ) -> Result<Schedule, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PATCH,
            &format!("/api/v1/schedules/{schedule_id}"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_schedule(
        &self,
        schedule_id: ignite_containers_core::ids::ScheduleId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/schedules/{schedule_id}"),
            None,
        )
        .await
    }

    pub async fn run_schedule(
        &self,
        schedule_id: ignite_containers_core::ids::ScheduleId,
    ) -> Result<Value, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/schedules/{schedule_id}/run"),
            Some(&body),
        )
        .await
    }

    // files ----------------------------------------------------------------

    pub async fn list_files(&self, server_id: ServerId, path: &str) -> Result<FileList, Error> {
        self.send(
            Method::GET,
            &format!(
                "/api/v1/servers/{server_id}/files?path={}",
                urlencoding(path)
            ),
            None,
        )
        .await
    }

    pub async fn read_file(&self, server_id: ServerId, path: &str) -> Result<FileContent, Error> {
        self.send(
            Method::GET,
            &format!(
                "/api/v1/servers/{server_id}/files/content?path={}",
                urlencoding(path)
            ),
            None,
        )
        .await
    }

    pub async fn write_file(
        &self,
        server_id: ServerId,
        request: &WriteFile,
    ) -> Result<FileEntry, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PUT,
            &format!("/api/v1/servers/{server_id}/files/content"),
            Some(&body),
        )
        .await
    }

    pub async fn mkdir_file(&self, server_id: ServerId, path: &str) -> Result<Value, Error> {
        let body = serde_json::json!({"path": path});
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{server_id}/files/mkdir"),
            Some(&body),
        )
        .await
    }

    pub async fn rename_file(
        &self,
        server_id: ServerId,
        from: &str,
        to: &str,
    ) -> Result<Value, Error> {
        let body = serde_json::json!({"from": from, "to": to});
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{server_id}/files/rename"),
            Some(&body),
        )
        .await
    }

    pub async fn compress_file(&self, server_id: ServerId, path: &str) -> Result<FileEntry, Error> {
        let body = serde_json::json!({"path": path});
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{server_id}/files/compress"),
            Some(&body),
        )
        .await
    }

    pub async fn decompress_file(&self, server_id: ServerId, path: &str) -> Result<Value, Error> {
        let body = serde_json::json!({"path": path});
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{server_id}/files/decompress"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_file(&self, server_id: ServerId, path: &str) -> Result<Value, Error> {
        self.send(
            Method::DELETE,
            &format!(
                "/api/v1/servers/{server_id}/files?path={}",
                urlencoding(path)
            ),
            None,
        )
        .await
    }

    pub async fn suspend_server(&self, id: ServerId) -> Result<Server, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{id}/suspend"),
            Some(&body),
        )
        .await
    }

    pub async fn resume_server(&self, id: ServerId) -> Result<Server, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{id}/resume"),
            Some(&body),
        )
        .await
    }

    // allocations ----------------------------------------------------------

    pub async fn list_allocations(
        &self,
        node_id: Option<NodeId>,
        server_id: Option<ServerId>,
    ) -> Result<Vec<Allocation>, Error> {
        let mut query = Vec::new();
        if let Some(node_id) = node_id {
            query.push(format!("node_id={node_id}"));
        }
        if let Some(server_id) = server_id {
            query.push(format!("server_id={server_id}"));
        }
        let path = if query.is_empty() {
            "/api/v1/allocations".to_string()
        } else {
            format!("/api/v1/allocations?{}", query.join("&"))
        };
        let list: AllocationList = self.send(Method::GET, &path, None).await?;
        Ok(list.allocations)
    }

    pub async fn create_allocations(
        &self,
        node_id: NodeId,
        request: &CreateAllocations,
    ) -> Result<AllocationResult, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/nodes/{node_id}/allocations"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_allocation(&self, id: AllocationId) -> Result<(), Error> {
        self.send_empty(Method::DELETE, &format!("/api/v1/allocations/{id}"), None)
            .await
    }

    // templates ------------------------------------------------------------

    pub async fn list_templates(&self) -> Result<Vec<TemplateRecord>, Error> {
        let list: TemplateList = self.send(Method::GET, "/api/v1/templates", None).await?;
        Ok(list.templates)
    }

    pub async fn get_template(&self, id: TemplateId) -> Result<TemplateRecord, Error> {
        self.send(Method::GET, &format!("/api/v1/templates/{id}"), None)
            .await
    }

    pub async fn import_egg(&self, request: &ImportEgg) -> Result<ImportResult, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/templates", Some(&body))
            .await
    }

    pub async fn set_auto_update(
        &self,
        id: TemplateId,
        policy: AutoUpdate,
    ) -> Result<TemplateRecord, Error> {
        let body = serde_json::json!({"auto_update": policy});
        self.send(
            Method::PATCH,
            &format!("/api/v1/templates/{id}"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_template(&self, id: TemplateId) -> Result<(), Error> {
        self.send_empty(Method::DELETE, &format!("/api/v1/templates/{id}"), None)
            .await
    }

    /// Submits a signed template feed (`ignite-containers.feed/v1`). Fetching the feed is the
    /// caller's responsibility; Ignite Containers never makes outbound requests on its own.
    pub async fn sync_template_feed(
        &self,
        feed: Value,
        signature: &str,
        public_key: &str,
        create_missing: bool,
    ) -> Result<FeedResult, Error> {
        let body = serde_json::json!({
            "feed": feed,
            "signature": signature,
            "public_key": public_key,
            "create_missing": create_missing,
        });
        self.send(Method::POST, "/api/v1/template-feeds", Some(&body))
            .await
    }

    // servers --------------------------------------------------------------

    pub async fn list_servers(&self, query: ServerQuery) -> Result<Vec<Server>, Error> {
        let mut parts = Vec::new();
        if let Some(node_id) = query.node_id {
            parts.push(format!("node_id={node_id}"));
        }
        if let Some(state) = query.state {
            let value = serde_json::to_value(state)?;
            if let Some(value) = value.as_str() {
                parts.push(format!("state={value}"));
            }
        }
        if let Some(external_ref) = query.external_ref {
            parts.push(format!("external_ref={}", urlencoding(&external_ref)));
        }
        if let Some(name) = query.name {
            parts.push(format!("name={}", urlencoding(&name)));
        }
        if let Some(limit) = query.limit {
            parts.push(format!("limit={limit}"));
        }
        if let Some(offset) = query.offset {
            parts.push(format!("offset={offset}"));
        }
        let path = if parts.is_empty() {
            "/api/v1/servers".to_string()
        } else {
            format!("/api/v1/servers?{}", parts.join("&"))
        };
        let list: ServerList = self.send(Method::GET, &path, None).await?;
        Ok(list.servers)
    }

    pub async fn create_server(&self, request: &CreateServer) -> Result<ServerDetail, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/servers", Some(&body))
            .await
    }

    pub async fn get_server(&self, id: ServerId) -> Result<ServerDetail, Error> {
        self.send(Method::GET, &format!("/api/v1/servers/{id}"), None)
            .await
    }

    pub async fn update_server(
        &self,
        id: ServerId,
        request: &UpdateServer,
    ) -> Result<Server, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::PATCH, &format!("/api/v1/servers/{id}"), Some(&body))
            .await
    }

    pub async fn delete_server(&self, id: ServerId) -> Result<(), Error> {
        self.send_empty(Method::DELETE, &format!("/api/v1/servers/{id}"), None)
            .await
    }

    pub async fn power(&self, id: ServerId, signal: PowerSignal) -> Result<Job, Error> {
        let body = serde_json::json!({"signal": signal});
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{id}/power"),
            Some(&body),
        )
        .await
    }

    pub async fn command(&self, id: ServerId, command: &str) -> Result<Value, Error> {
        let body = serde_json::json!({"command": command});
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{id}/command"),
            Some(&body),
        )
        .await
    }

    pub async fn logs(
        &self,
        id: ServerId,
        lines: u32,
    ) -> Result<ignite_containers_proto::LogChunk, Error> {
        self.send(
            Method::GET,
            &format!("/api/v1/servers/{id}/logs?lines={}", lines.clamp(1, 2000)),
            None,
        )
        .await
    }

    pub async fn stats(&self, id: ServerId) -> Result<ignite_containers_proto::ServerStats, Error> {
        self.send(Method::GET, &format!("/api/v1/servers/{id}/stats"), None)
            .await
    }

    pub async fn attach_allocation(
        &self,
        id: ServerId,
        allocation_id: AllocationId,
    ) -> Result<Allocation, Error> {
        let body = serde_json::json!({"allocation_id": allocation_id});
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{id}/allocations"),
            Some(&body),
        )
        .await
    }

    pub async fn detach_allocation(
        &self,
        id: ServerId,
        allocation_id: AllocationId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/servers/{id}/allocations/{allocation_id}"),
            None,
        )
        .await
    }

    pub async fn reinstall(&self, id: ServerId) -> Result<Job, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{id}/reinstall"),
            Some(&body),
        )
        .await
    }

    pub async fn list_jobs(&self, id: ServerId) -> Result<Vec<Job>, Error> {
        let list: JobList = self
            .send(Method::GET, &format!("/api/v1/servers/{id}/jobs"), None)
            .await?;
        Ok(list.jobs)
    }

    pub async fn get_server_environment(
        &self,
        id: ServerId,
    ) -> Result<BTreeMap<String, String>, Error> {
        let body: ServerEnvironment = self
            .send(
                Method::GET,
                &format!("/api/v1/servers/{id}/environment"),
                None,
            )
            .await?;
        Ok(body.environment)
    }

    pub async fn list_audit_events(&self, limit: i64) -> Result<Vec<AuditEvent>, Error> {
        let body: AuditList = self
            .send(
                Method::GET,
                &format!("/api/v1/audit-events?limit={}", limit.clamp(1, 1000)),
                None,
            )
            .await?;
        Ok(body.events)
    }

    pub async fn get_node_state(&self, id: NodeId) -> Result<NodeState, Error> {
        self.send(Method::GET, &format!("/api/v1/nodes/{id}/state"), None)
            .await
    }

    pub async fn get_database_host(&self, id: DatabaseHostId) -> Result<DatabaseHost, Error> {
        self.send(Method::GET, &format!("/api/v1/database-hosts/{id}"), None)
            .await
    }

    pub async fn list_template_versions(
        &self,
        id: TemplateId,
    ) -> Result<Vec<TemplateVersionRecord>, Error> {
        let body: TemplateVersionList = self
            .send(
                Method::GET,
                &format!("/api/v1/templates/{id}/versions"),
                None,
            )
            .await?;
        Ok(body.versions)
    }

    pub async fn publish_template_version(
        &self,
        id: TemplateId,
        request: &PublishTemplateVersion,
    ) -> Result<TemplateRecord, Error> {
        self.send(
            Method::POST,
            &format!("/api/v1/templates/{id}/versions"),
            Some(&serde_json::to_value(request)?),
        )
        .await
    }

    pub async fn list_proxy_routes(&self) -> Result<Vec<ProxyRoute>, Error> {
        let body: ProxyRouteList = self.send(Method::GET, "/api/v1/proxy/routes", None).await?;
        Ok(body.routes)
    }

    pub async fn create_proxy_route(
        &self,
        request: &CreateProxyRoute,
    ) -> Result<ProxyMutation, Error> {
        self.send(
            Method::POST,
            "/api/v1/proxy/routes",
            Some(&serde_json::to_value(request)?),
        )
        .await
    }

    pub async fn update_proxy_route(
        &self,
        id: uuid::Uuid,
        request: &UpdateProxyRoute,
    ) -> Result<ProxyMutation, Error> {
        self.send(
            Method::PATCH,
            &format!("/api/v1/proxy/routes/{id}"),
            Some(&serde_json::to_value(request)?),
        )
        .await
    }

    pub async fn delete_proxy_route(&self, id: uuid::Uuid) -> Result<ProxyDeleteResult, Error> {
        self.send(Method::DELETE, &format!("/api/v1/proxy/routes/{id}"), None)
            .await
    }

    pub async fn proxy_status(&self) -> Result<ProxyStatus, Error> {
        self.send(Method::GET, "/api/v1/proxy/status", None).await
    }

    pub async fn reconcile_proxy(&self) -> Result<ProxyReconcileResult, Error> {
        self.send(Method::POST, "/api/v1/proxy/reconcile", None)
            .await
    }

    pub async fn chmod_file(
        &self,
        server_id: ServerId,
        path: &str,
        mode: &str,
    ) -> Result<Value, Error> {
        self.send(
            Method::POST,
            &format!("/api/v1/servers/{server_id}/files/chmod"),
            Some(&serde_json::json!({ "path": path, "mode": mode })),
        )
        .await
    }

    /// Open the authenticated SSE log stream. The caller owns the response and
    /// must consume/close the byte stream.
    pub async fn logs_stream(&self, id: ServerId) -> Result<reqwest::Response, Error> {
        let response = self
            .stream_http
            .get(format!("{}/api/v1/servers/{id}/logs/stream", self.base_url))
            .bearer_auth(&self.api_key)
            .header("accept", "text/event-stream")
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await?;
            return Err(parse_error(status, &text));
        }
        Ok(response)
    }

    pub async fn health(&self) -> Result<Value, Error> {
        self.send(Method::GET, "/healthz", None).await
    }

    pub async fn ready(&self) -> Result<Value, Error> {
        self.send(Method::GET, "/readyz", None).await
    }

    pub async fn openapi(&self) -> Result<Value, Error> {
        self.send(Method::GET, "/openapi.json", None).await
    }

    pub async fn docs(&self) -> Result<String, Error> {
        let response = self
            .http
            .get(format!("{}/docs", self.base_url))
            .bearer_auth(&self.api_key)
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(parse_error(status, &text));
        }
        Ok(text)
    }

    pub async fn metrics(&self) -> Result<String, Error> {
        let response = self
            .http
            .get(format!("{}/metrics", self.base_url))
            .bearer_auth(&self.api_key)
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(parse_error(status, &text));
        }
        Ok(text)
    }

    /// Escape hatch for endpoints that are added before this client is regenerated.
    pub async fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Result<T, Error> {
        let method = Method::from_bytes(method.to_uppercase().as_bytes())
            .map_err(|_| Error::InvalidUrl(method.to_string()))?;
        self.send(method, path, body.as_ref()).await
    }
}

fn parse_error(status: StatusCode, text: &str) -> Error {
    match serde_json::from_str::<ErrorEnvelope>(text) {
        Ok(envelope) => Error::Api {
            status: status.as_u16(),
            code: envelope.error.code,
            message: envelope.error.message,
            details: envelope.error.details,
        },
        Err(_) => Error::Api {
            status: status.as_u16(),
            code: "unknown".to_string(),
            message: text.chars().take(500).collect(),
            details: None,
        },
    }
}

fn urlencoding(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}
