//! Async Rust client for the Ignite Containers control plane.
//!
//! ```no_run
//! # async fn run() -> Result<(), ignite_containers_client::Error> {
//! let client = ignite_containers_client::Client::new("https://games.example.com", "ic_...")?;
//! let pods = client.list_pods(ignite_containers_client::PodQuery::default()).await?;
//! println!("{} pods", pods.len());
//! # Ok(())
//! # }
//! ```

use chrono::{DateTime, Utc};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::Duration;

mod types;
pub use types::*;

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
pub struct PodList {
    pub pods: Vec<Pod>,
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
    pub pod_ids: Vec<PodId>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseHostList {
    pub hosts: Vec<DatabaseHost>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseList {
    pub databases: Vec<PodDatabase>,
}

#[derive(Debug, Deserialize)]
pub struct CreatedDatabase {
    #[serde(flatten)]
    pub database: PodDatabase,
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
    pub pod_id: PodId,
    pub success: bool,
    pub error: Option<String>,
    pub job_id: Option<JobId>,
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
pub struct PodEnvironment {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pod_id: Option<PodId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_port: Option<u16>,
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
    pub pod_id: Option<PodId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_port: Option<u16>,
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
    pub certificates: usize,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyReconcileResult {
    pub configured: bool,
    pub applied: bool,
    pub routes: usize,
    #[serde(default)]
    pub certificates: usize,
    #[serde(default)]
    pub warning: Option<String>,
    #[serde(default)]
    pub caddy: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyCertificateList {
    pub certificates: Vec<ProxyCertificate>,
}

#[derive(Debug, Serialize)]
pub struct UploadProxyCertificate {
    pub domain: String,
    pub certificate_pem: String,
    pub private_key_pem: String,
}

#[derive(Debug, Deserialize)]
pub struct ProxyCertificateMutation {
    pub certificate: ProxyCertificate,
    pub replaced: bool,
    pub applied: bool,
    #[serde(default)]
    pub warning: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyCertificateDeleteResult {
    pub deleted: bool,
    pub applied: bool,
    #[serde(default)]
    pub warning: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_group_id: Option<RuntimeGroupId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dedicated_tenant_id: Option<TenantId>,
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
            runtime_group_id: None,
            dedicated_tenant_id: None,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_group_id: Option<RuntimeGroupId>,
    #[serde(default)]
    pub clear_runtime_group: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedicated_tenant_id: Option<TenantId>,
    #[serde(default)]
    pub clear_dedicated_tenant: bool,
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

impl CreatePod {
    #[must_use]
    pub fn new(node_id: NodeId, template_id: TemplateId, name: impl Into<String>) -> Self {
        Self {
            node_id: Some(node_id),
            template_id,
            runtime_group_id: None,
            auto_start: false,
            template_version: None,
            name: name.into(),
            description: String::new(),
            external_ref: None,
            idempotency_key: None,
            cap_add: Vec::new(),
            devices: Vec::new(),
            labels: BTreeMap::new(),
            image: None,
            variables: BTreeMap::new(),
            limits: None,
            allocations: Vec::new(),
            auto_allocations: Some(1),
            protocol: None,
        }
    }

    /// Place the workload on any online node in a runtime group.
    #[must_use]
    pub fn in_runtime_group(
        runtime_group_id: RuntimeGroupId,
        template_id: TemplateId,
        name: impl Into<String>,
    ) -> Self {
        Self {
            node_id: None,
            template_id,
            runtime_group_id: Some(runtime_group_id),
            auto_start: false,
            template_version: None,
            name: name.into(),
            description: String::new(),
            external_ref: None,
            idempotency_key: None,
            cap_add: Vec::new(),
            devices: Vec::new(),
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
pub struct CreatePod {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
    pub template_id: TemplateId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_group_id: Option<RuntimeGroupId>,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_version: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cap_add: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub devices: Vec<String>,
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
pub struct UpdatePod {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_add: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devices: Option<Vec<String>>,
}

#[derive(Debug, Default)]
pub struct PodQuery {
    pub node_id: Option<NodeId>,
    pub state: Option<PodState>,
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
    pub pod_ids: Vec<PodId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CreateVnet {
    pub name: String,
    pub cidr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct AttachVnet {
    pub pod_id: PodId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateInterconnect {
    pub name: String,
    pub vnet_id: VnetId,
    #[serde(default)]
    pub pod_ids: Vec<PodId>,
}

#[derive(Debug, Serialize)]
pub struct CreateListener {
    pub name: String,
    pub node_id: NodeId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind_ip: Option<String>,
    pub port: u16,
    pub protocol: String,
    pub adapter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub real_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub velocity_secret: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateListenerRoute {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pod_id: Option<PodId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_start: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offline_motd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_timeout_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ExportTemplate {
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub version: String,
    #[serde(default)]
    pub include_files: bool,
}

#[derive(Debug, Serialize)]
pub struct CloneTemplate {
    pub node_id: NodeId,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<Limits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_allocations: Option<u8>,
    #[serde(default)]
    pub restore_files: bool,
}

#[derive(Debug, Deserialize)]
pub struct VnetList {
    pub vnets: Vec<crate::types::Vnet>,
}

#[derive(Debug, Deserialize)]
pub struct VnetAttachmentList {
    pub attachments: Vec<crate::types::VnetAttachment>,
}

#[derive(Debug, Deserialize)]
pub struct InterconnectList {
    pub interconnects: Vec<crate::types::Interconnect>,
}

#[derive(Debug, Deserialize)]
pub struct ListenerList {
    pub listeners: Vec<crate::types::ProxyListener>,
}

#[derive(Debug, Deserialize)]
pub struct ListenerRouteList {
    pub routes: Vec<crate::types::ProxyListenerRoute>,
}

#[derive(Debug, Deserialize)]
pub struct ExportTemplateResult {
    pub record: crate::types::TemplateRecord,
    #[serde(default)]
    pub backup_id: Option<uuid::Uuid>,
    #[serde(default)]
    pub snapshot_ready: bool,
}

#[derive(Debug, Deserialize)]
pub struct CloneTemplateResult {
    pub pod: crate::types::Pod,
    pub allocations: Vec<crate::types::Allocation>,
    #[serde(default)]
    pub files_restored: bool,
    #[serde(default)]
    pub warning: Option<String>,
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
pub struct CreatePodDatabase {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_id: Option<DatabaseHostId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateRuntimeGroup {
    pub slug: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<TenantId>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateRuntimeGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<TenantId>,
    #[serde(default)]
    pub clear_tenant: bool,
}

#[derive(Debug, Serialize)]
pub struct ImportNest {
    pub nest: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<Channel>,
    pub save: bool,
}

impl ImportNest {
    #[must_use]
    pub fn new(nest: Value) -> Self {
        Self {
            nest,
            version: None,
            channel: None,
            save: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NestImportFailure {
    pub name: String,
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct NestImportResult {
    pub saved: bool,
    #[serde(default)]
    pub imported: Vec<ImportResult>,
    #[serde(default)]
    pub failed: Vec<NestImportFailure>,
}

#[derive(Debug, Deserialize)]
struct RuntimeGroupList {
    groups: Vec<RuntimeGroup>,
}

#[derive(Debug, Deserialize)]
struct IpAddressList {
    addresses: Vec<IpAddress>,
}

#[derive(Debug, Deserialize)]
struct PortForwardList {
    forwards: Vec<PortForward>,
}

#[derive(Debug, Deserialize)]
struct RevisionList {
    revisions: Vec<Revision>,
}

#[derive(Debug, Deserialize)]
struct RolloutStatus {
    #[serde(default)]
    rollout: Option<Rollout>,
}

#[derive(Debug, Deserialize)]
struct GlobalJobList {
    jobs: Vec<Job>,
}

#[derive(Debug, Deserialize)]
struct WebhookList {
    webhooks: Vec<WebhookSubscription>,
}

#[derive(Debug, Deserialize)]
struct WebhookDeliveryList {
    deliveries: Vec<WebhookDelivery>,
}

#[derive(Debug, Deserialize)]
struct RotatedWebhookSecret {
    secret: String,
}

#[derive(Debug, Deserialize)]
struct PodPortList {
    ports: Vec<PodPort>,
}

#[derive(Debug, Deserialize)]
struct PortHistory {
    events: Vec<PodPortEvent>,
}

#[derive(Debug, Deserialize)]
struct ImageList {
    images: Vec<Image>,
}

#[derive(Debug, Deserialize)]
struct BuildList {
    builds: Vec<GitBuild>,
}

#[derive(Debug, Deserialize)]
struct MetricsArchiveList {
    archives: Vec<MetricsArchive>,
}

#[derive(Debug, Deserialize)]
struct BackendList {
    backends: Vec<StorageBackend>,
}

#[derive(Debug, Deserialize)]
struct VolumeList {
    volumes: Vec<Volume>,
}

#[derive(Debug, Deserialize)]
struct MountList {
    mounts: Vec<PodVolumeMountView>,
}

#[derive(Debug, Deserialize)]
struct ExternalDatabaseList {
    databases: Vec<ExternalDatabase>,
}

#[derive(Debug, Deserialize)]
struct ExternalDatabaseBindingList {
    bindings: Vec<PodExternalDatabaseView>,
}

#[derive(Debug, Deserialize)]
struct StorageGatewayTokenList {
    tokens: Vec<StorageGatewayToken>,
}

#[derive(Debug, Serialize)]
pub struct CreateTenant {
    pub name: String,
    pub slug: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<ApiKeyLimits>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateTenant {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TenantStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(default)]
    pub clear_reference: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<ApiKeyLimits>,
}

#[derive(Debug, Serialize)]
pub struct CreateResourceGroup {
    pub name: String,
    pub slug: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateResourceGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(default)]
    pub clear_reference: bool,
}

#[derive(Debug, Serialize)]
pub struct CreateTransferKey {
    pub tenant_id: uuid::Uuid,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct StartTransfer {
    pub transfer_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateMeteringProfile {
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<uuid::Uuid>,
    pub formula: String,
    #[serde(default)]
    pub bill_internal_traffic: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_from: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CreatePodPort {
    pub container_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<PortProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposure: Option<PortExposure>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdatePodPort {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<PortProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposure: Option<PortExposure>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub clear_label: bool,
}

#[derive(Debug, Serialize)]
pub struct PullImage {
    pub reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    #[serde(default)]
    pub bytes: u64,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UploadImage {
    pub reference: String,
    pub digest: String,
    #[serde(default)]
    pub stored_bytes: u64,
    #[serde(default)]
    pub compressed_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateGitCredential {
    pub tenant_id: TenantId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    pub name: String,
    pub provider: GitProvider,
    pub auth: GitAuthKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateGitSource {
    pub resource_group_id: ResourceGroupId,
    pub pod_id: PodId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_id: Option<GitCredentialId>,
    pub name: String,
    pub provider: GitProvider,
    pub repo: String,
    pub reference: String,
    pub ref_kind: GitRefKind,
    pub dockerfile: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub context: String,
    #[serde(default)]
    pub build_args: BTreeMap<String, String>,
    pub auto_update: bool,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateGitSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<GitProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_kind: Option<GitRefKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dockerfile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_args: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_update: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct TriggerGitBuild {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpsertMetricsStoragePolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_retention_days: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_after_days: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_retention_days: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_buffer_minutes: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletion_grace_days: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3_bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3_region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3_access_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3_secret_key: Option<String>,
    #[serde(default)]
    pub confirm_short_grace: bool,
}

#[derive(Debug, Serialize)]
pub struct DeployCompose {
    pub compose: String,
    pub template_id: TemplateId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_version: Option<String>,
    pub resource_group_id: ResourceGroupId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_group_id: Option<RuntimeGroupId>,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<Limits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_size_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_backend_id: Option<StorageBackendId>,
    #[serde(default)]
    pub network_cidrs: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct CreateStorageBackend {
    pub tenant_id: TenantId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    pub name: String,
    pub kind: StorageBackendKind,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub options: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateStorageBackend {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<StorageBackendKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    #[serde(default)]
    pub clear_resource_group: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(default)]
    pub clear_reference: bool,
}

#[derive(Debug, Serialize)]
pub struct CreateVolume {
    pub resource_group_id: ResourceGroupId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_id: Option<StorageBackendId>,
    pub storage_class: VolumeStorageClass,
    pub name: String,
    pub size_mb: u64,
    pub max_size_mb: u64,
    pub auto_scale: bool,
    pub access_mode: VolumeAccessMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateVolume {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_scale: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<VolumeStorageClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_mode: Option<VolumeAccessMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_id: Option<StorageBackendId>,
    #[serde(default)]
    pub clear_backend: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(default)]
    pub clear_reference: bool,
}

#[derive(Debug, Serialize)]
pub struct CreateStorageGatewayToken {
    pub tenant_id: TenantId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_id: Option<StorageBackendId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_id: Option<VolumeId>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(default)]
    pub read_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in_days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ReportVolumeUsage {
    pub used_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct MountVolume {
    pub volume_id: VolumeId,
    pub mount_path: String,
    #[serde(default)]
    pub read_only: bool,
}

#[derive(Debug, Serialize)]
pub struct CreateExternalDatabase {
    pub tenant_id: TenantId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    pub name: String,
    pub engine: ExternalDatabaseEngine,
    pub host: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default)]
    pub options: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateExternalDatabase {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<ExternalDatabaseEngine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    #[serde(default)]
    pub clear_resource_group: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(default)]
    pub clear_reference: bool,
}

#[derive(Debug, Serialize)]
pub struct BindExternalDatabase {
    pub database_id: ExternalDatabaseId,
    pub env_prefix: String,
}

#[derive(Debug, Serialize)]
pub struct GrantTenantAccess {
    pub api_key_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
struct TenantAccessList {
    api_key_ids: Vec<uuid::Uuid>,
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

    async fn send_bytes(&self, path: &str) -> Result<Vec<u8>, Error> {
        let response = self
            .http
            .get(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.api_key)
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await?;
            return Err(parse_error(status, &text));
        }
        Ok(response.bytes().await?.to_vec())
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

    pub async fn node_updates(&self, id: NodeId) -> Result<HostUpdateStatus, Error> {
        self.send(Method::GET, &format!("/api/v1/nodes/{id}/updates"), None)
            .await
    }

    pub async fn check_node_updates(&self, id: NodeId) -> Result<HostUpdateStatus, Error> {
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

    pub async fn delete_database_host(&self, id: DatabaseHostId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/database-hosts/{id}"),
            None,
        )
        .await
    }

    pub async fn test_database_host(&self, id: DatabaseHostId) -> Result<DatabaseHostTest, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/database-hosts/{id}/test"),
            Some(&body),
        )
        .await
    }

    pub async fn list_pod_databases(&self, pod_id: PodId) -> Result<Vec<PodDatabase>, Error> {
        let list: DatabaseList = self
            .send(
                Method::GET,
                &format!("/api/v1/pods/{pod_id}/databases"),
                None,
            )
            .await?;
        Ok(list.databases)
    }

    pub async fn create_pod_database(
        &self,
        pod_id: PodId,
        request: &CreatePodDatabase,
    ) -> Result<CreatedDatabase, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{pod_id}/databases"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_pod_database(
        &self,
        pod_id: PodId,
        database_id: PodDatabaseId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/pods/{pod_id}/databases/{database_id}"),
            None,
        )
        .await
    }

    // sftp -----------------------------------------------------------------

    pub async fn create_sftp_credential(
        &self,
        pod_id: PodId,
        ttl_seconds: Option<i64>,
    ) -> Result<CreatedSftp, Error> {
        let body = serde_json::json!({"ttl_seconds": ttl_seconds});
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{pod_id}/sftp"),
            Some(&body),
        )
        .await
    }

    pub async fn list_sftp_credentials(&self, pod_id: PodId) -> Result<Vec<SftpCredential>, Error> {
        let list: SftpList = self
            .send(Method::GET, &format!("/api/v1/pods/{pod_id}/sftp"), None)
            .await?;
        Ok(list.credentials)
    }

    pub async fn revoke_sftp_credential(
        &self,
        pod_id: PodId,
        credential_id: SftpCredentialId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/pods/{pod_id}/sftp/{credential_id}"),
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

    pub async fn retry_email(&self, id: EmailId) -> Result<EmailMessage, Error> {
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
        pod_ids: Vec<PodId>,
        signal: PowerSignal,
    ) -> Result<Vec<BulkResult>, Error> {
        let body = serde_json::json!({"pod_ids": pod_ids, "signal": signal});
        let response: BulkResponse = self
            .send(Method::POST, "/api/v1/pods/bulk/power", Some(&body))
            .await?;
        Ok(response.results)
    }

    pub async fn bulk_command(
        &self,
        pod_ids: Vec<PodId>,
        command: &str,
    ) -> Result<Vec<BulkResult>, Error> {
        let body = serde_json::json!({"pod_ids": pod_ids, "command": command});
        let response: BulkResponse = self
            .send(Method::POST, "/api/v1/pods/bulk/command", Some(&body))
            .await?;
        Ok(response.results)
    }

    // backups --------------------------------------------------------------

    pub async fn list_backups(&self, pod_id: PodId) -> Result<Vec<Backup>, Error> {
        let list: BackupList = self
            .send(Method::GET, &format!("/api/v1/pods/{pod_id}/backups"), None)
            .await?;
        Ok(list.backups)
    }

    pub async fn create_backup(
        &self,
        pod_id: PodId,
        request: &CreateBackup,
    ) -> Result<Backup, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{pod_id}/backups"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_backup(&self, pod_id: PodId, backup_id: BackupId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/pods/{pod_id}/backups/{backup_id}"),
            None,
        )
        .await
    }

    pub async fn restore_backup(
        &self,
        pod_id: PodId,
        backup_id: BackupId,
    ) -> Result<Backup, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{pod_id}/backups/{backup_id}/restore"),
            Some(&body),
        )
        .await
    }

    pub async fn download_backup(
        &self,
        pod_id: PodId,
        backup_id: BackupId,
    ) -> Result<Vec<u8>, Error> {
        let response = self
            .http
            .get(format!(
                "{}/api/v1/pods/{pod_id}/backups/{backup_id}/download",
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

    /// Streams the data volume (or one subdirectory) as a gzip archive. The
    /// caller owns the response and must consume/close the byte stream.
    pub async fn export_volume(
        &self,
        pod_id: PodId,
        path: Option<&str>,
    ) -> Result<reqwest::Response, Error> {
        let mut url = format!("{}/api/v1/pods/{pod_id}/volume/export", self.base_url);
        if let Some(path) = path.filter(|value| !value.is_empty()) {
            url.push_str(&format!("?path={}", urlencoding(path)));
        }
        let response = self
            .stream_http
            .get(url)
            .bearer_auth(&self.api_key)
            .header("accept", "application/gzip")
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(parse_error(status, &text));
        }
        Ok(response)
    }

    /// Uploads a gzip archive into the data volume (or one subdirectory).
    /// `wipe` clears the target directory before extracting.
    pub async fn import_volume(
        &self,
        pod_id: PodId,
        path: Option<&str>,
        wipe: bool,
        archive: Vec<u8>,
    ) -> Result<Value, Error> {
        let response = self
            .http
            .post(format!(
                "{}/api/v1/pods/{pod_id}/volume/import",
                self.base_url
            ))
            .bearer_auth(&self.api_key)
            .header("content-type", "application/gzip")
            .query(&[
                ("path", path.unwrap_or_default()),
                ("wipe", if wipe { "true" } else { "false" }),
            ])
            .body(archive)
            .timeout(std::time::Duration::from_secs(3600))
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(parse_error(status, &text));
        }
        Ok(serde_json::from_str(&text)?)
    }

    // schedules ------------------------------------------------------------

    pub async fn list_schedules(&self, pod_id: PodId) -> Result<Vec<Schedule>, Error> {
        let list: ScheduleList = self
            .send(
                Method::GET,
                &format!("/api/v1/pods/{pod_id}/schedules"),
                None,
            )
            .await?;
        Ok(list.schedules)
    }

    pub async fn create_schedule(
        &self,
        pod_id: PodId,
        request: &CreateSchedule,
    ) -> Result<Schedule, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{pod_id}/schedules"),
            Some(&body),
        )
        .await
    }

    pub async fn update_schedule(
        &self,
        schedule_id: ScheduleId,
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

    pub async fn delete_schedule(&self, schedule_id: ScheduleId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/schedules/{schedule_id}"),
            None,
        )
        .await
    }

    pub async fn run_schedule(&self, schedule_id: ScheduleId) -> Result<Value, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/schedules/{schedule_id}/run"),
            Some(&body),
        )
        .await
    }

    // files ----------------------------------------------------------------

    pub async fn list_files(&self, pod_id: PodId, path: &str) -> Result<FileList, Error> {
        self.send(
            Method::GET,
            &format!("/api/v1/pods/{pod_id}/files?path={}", urlencoding(path)),
            None,
        )
        .await
    }

    pub async fn read_file(&self, pod_id: PodId, path: &str) -> Result<FileContent, Error> {
        self.send(
            Method::GET,
            &format!(
                "/api/v1/pods/{pod_id}/files/content?path={}",
                urlencoding(path)
            ),
            None,
        )
        .await
    }

    pub async fn write_file(&self, pod_id: PodId, request: &WriteFile) -> Result<FileEntry, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PUT,
            &format!("/api/v1/pods/{pod_id}/files/content"),
            Some(&body),
        )
        .await
    }

    pub async fn mkdir_file(&self, pod_id: PodId, path: &str) -> Result<Value, Error> {
        let body = serde_json::json!({"path": path});
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{pod_id}/files/mkdir"),
            Some(&body),
        )
        .await
    }

    pub async fn rename_file(&self, pod_id: PodId, from: &str, to: &str) -> Result<Value, Error> {
        let body = serde_json::json!({"from": from, "to": to});
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{pod_id}/files/rename"),
            Some(&body),
        )
        .await
    }

    pub async fn compress_file(&self, pod_id: PodId, path: &str) -> Result<FileEntry, Error> {
        let body = serde_json::json!({"path": path});
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{pod_id}/files/compress"),
            Some(&body),
        )
        .await
    }

    pub async fn decompress_file(&self, pod_id: PodId, path: &str) -> Result<Value, Error> {
        let body = serde_json::json!({"path": path});
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{pod_id}/files/decompress"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_file(&self, pod_id: PodId, path: &str) -> Result<Value, Error> {
        self.send(
            Method::DELETE,
            &format!("/api/v1/pods/{pod_id}/files?path={}", urlencoding(path)),
            None,
        )
        .await
    }

    pub async fn suspend_pod(&self, id: PodId) -> Result<Pod, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{id}/suspend"),
            Some(&body),
        )
        .await
    }

    pub async fn resume_pod(&self, id: PodId) -> Result<Pod, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{id}/resume"),
            Some(&body),
        )
        .await
    }

    // allocations ----------------------------------------------------------

    pub async fn list_allocations(
        &self,
        node_id: Option<NodeId>,
        pod_id: Option<PodId>,
    ) -> Result<Vec<Allocation>, Error> {
        let mut query = Vec::new();
        if let Some(node_id) = node_id {
            query.push(format!("node_id={node_id}"));
        }
        if let Some(pod_id) = pod_id {
            query.push(format!("pod_id={pod_id}"));
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

    // pods --------------------------------------------------------------

    pub async fn list_pods(&self, query: PodQuery) -> Result<Vec<Pod>, Error> {
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
            "/api/v1/pods".to_string()
        } else {
            format!("/api/v1/pods?{}", parts.join("&"))
        };
        let list: PodList = self.send(Method::GET, &path, None).await?;
        Ok(list.pods)
    }

    pub async fn create_pod(&self, request: &CreatePod) -> Result<PodDetail, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/pods", Some(&body)).await
    }

    pub async fn get_pod(&self, id: PodId) -> Result<PodDetail, Error> {
        self.send(Method::GET, &format!("/api/v1/pods/{id}"), None)
            .await
    }

    pub async fn update_pod(&self, id: PodId, request: &UpdatePod) -> Result<Pod, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::PATCH, &format!("/api/v1/pods/{id}"), Some(&body))
            .await
    }

    pub async fn delete_pod(&self, id: PodId) -> Result<(), Error> {
        self.send_empty(Method::DELETE, &format!("/api/v1/pods/{id}"), None)
            .await
    }

    pub async fn power(&self, id: PodId, signal: PowerSignal) -> Result<Job, Error> {
        let body = serde_json::json!({"signal": signal});
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{id}/power"),
            Some(&body),
        )
        .await
    }

    pub async fn command(&self, id: PodId, command: &str) -> Result<Value, Error> {
        let body = serde_json::json!({"command": command});
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{id}/command"),
            Some(&body),
        )
        .await
    }

    pub async fn logs(&self, id: PodId, lines: u32) -> Result<LogChunk, Error> {
        self.send(
            Method::GET,
            &format!("/api/v1/pods/{id}/logs?lines={}", lines.clamp(1, 2000)),
            None,
        )
        .await
    }

    pub async fn stats(&self, id: PodId) -> Result<PodStats, Error> {
        self.send(Method::GET, &format!("/api/v1/pods/{id}/stats"), None)
            .await
    }

    pub async fn attach_allocation(
        &self,
        id: PodId,
        allocation_id: AllocationId,
    ) -> Result<Allocation, Error> {
        let body = serde_json::json!({"allocation_id": allocation_id});
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{id}/allocations"),
            Some(&body),
        )
        .await
    }

    pub async fn detach_allocation(
        &self,
        id: PodId,
        allocation_id: AllocationId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/pods/{id}/allocations/{allocation_id}"),
            None,
        )
        .await
    }

    pub async fn reinstall(&self, id: PodId) -> Result<Job, Error> {
        let body = serde_json::json!({});
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{id}/reinstall"),
            Some(&body),
        )
        .await
    }

    pub async fn list_jobs(&self, id: PodId) -> Result<Vec<Job>, Error> {
        let list: JobList = self
            .send(Method::GET, &format!("/api/v1/pods/{id}/jobs"), None)
            .await?;
        Ok(list.jobs)
    }

    pub async fn get_pod_environment(&self, id: PodId) -> Result<BTreeMap<String, String>, Error> {
        let body: PodEnvironment = self
            .send(Method::GET, &format!("/api/v1/pods/{id}/environment"), None)
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

    pub async fn list_proxy_certificates(&self) -> Result<Vec<ProxyCertificate>, Error> {
        let body: ProxyCertificateList = self
            .send(Method::GET, "/api/v1/proxy/certificates", None)
            .await?;
        Ok(body.certificates)
    }

    pub async fn upload_proxy_certificate(
        &self,
        request: &UploadProxyCertificate,
    ) -> Result<ProxyCertificateMutation, Error> {
        self.send(
            Method::POST,
            "/api/v1/proxy/certificates",
            Some(&serde_json::to_value(request)?),
        )
        .await
    }

    pub async fn get_proxy_certificate(&self, id: uuid::Uuid) -> Result<ProxyCertificate, Error> {
        self.send(
            Method::GET,
            &format!("/api/v1/proxy/certificates/{id}"),
            None,
        )
        .await
    }

    pub async fn delete_proxy_certificate(
        &self,
        id: uuid::Uuid,
    ) -> Result<ProxyCertificateDeleteResult, Error> {
        self.send(
            Method::DELETE,
            &format!("/api/v1/proxy/certificates/{id}"),
            None,
        )
        .await
    }

    pub async fn list_vnets(&self) -> Result<Vec<crate::types::Vnet>, Error> {
        let body: VnetList = self.send(Method::GET, "/api/v1/vnets", None).await?;
        Ok(body.vnets)
    }

    pub async fn create_vnet(&self, request: &CreateVnet) -> Result<crate::types::Vnet, Error> {
        self.send(
            Method::POST,
            "/api/v1/vnets",
            Some(&serde_json::to_value(request)?),
        )
        .await
    }

    pub async fn attach_vnet(
        &self,
        vnet: VnetId,
        request: &AttachVnet,
    ) -> Result<crate::types::VnetAttachment, Error> {
        self.send(
            Method::POST,
            &format!("/api/v1/vnets/{vnet}/attachments"),
            Some(&serde_json::to_value(request)?),
        )
        .await
    }

    pub async fn list_interconnects(&self) -> Result<Vec<crate::types::Interconnect>, Error> {
        let body: InterconnectList = self
            .send(Method::GET, "/api/v1/interconnects", None)
            .await?;
        Ok(body.interconnects)
    }

    pub async fn create_interconnect(
        &self,
        request: &CreateInterconnect,
    ) -> Result<crate::types::Interconnect, Error> {
        self.send(
            Method::POST,
            "/api/v1/interconnects",
            Some(&serde_json::to_value(request)?),
        )
        .await
    }

    pub async fn list_listeners(&self) -> Result<Vec<crate::types::ProxyListener>, Error> {
        let body: ListenerList = self
            .send(Method::GET, "/api/v1/proxy/listeners", None)
            .await?;
        Ok(body.listeners)
    }

    pub async fn create_listener(
        &self,
        request: &CreateListener,
    ) -> Result<crate::types::ProxyListener, Error> {
        self.send(
            Method::POST,
            "/api/v1/proxy/listeners",
            Some(&serde_json::to_value(request)?),
        )
        .await
    }

    pub async fn create_listener_route(
        &self,
        listener: uuid::Uuid,
        request: &CreateListenerRoute,
    ) -> Result<crate::types::ProxyListenerRoute, Error> {
        self.send(
            Method::POST,
            &format!("/api/v1/proxy/listeners/{listener}/routes"),
            Some(&serde_json::to_value(request)?),
        )
        .await
    }

    pub async fn export_template(
        &self,
        pod: PodId,
        request: &ExportTemplate,
    ) -> Result<ExportTemplateResult, Error> {
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{pod}/export-template"),
            Some(&serde_json::to_value(request)?),
        )
        .await
    }

    pub async fn clone_template(
        &self,
        template: TemplateId,
        request: &CloneTemplate,
    ) -> Result<CloneTemplateResult, Error> {
        self.send(
            Method::POST,
            &format!("/api/v1/templates/{template}/clone"),
            Some(&serde_json::to_value(request)?),
        )
        .await
    }

    pub async fn chmod_file(&self, pod_id: PodId, path: &str, mode: &str) -> Result<Value, Error> {
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{pod_id}/files/chmod"),
            Some(&serde_json::json!({ "path": path, "mode": mode })),
        )
        .await
    }

    /// Open the authenticated SSE log stream. The caller owns the response and
    /// must consume/close the byte stream.
    pub async fn logs_stream(&self, id: PodId) -> Result<reqwest::Response, Error> {
        let response = self
            .stream_http
            .get(format!("{}/api/v1/pods/{id}/logs/stream", self.base_url))
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

    // runtime groups -------------------------------------------------------

    pub async fn list_deployment_classes(&self) -> Result<Vec<RuntimeGroup>, Error> {
        let list: RuntimeGroupList = self
            .send(Method::GET, "/api/v1/deployment-classes", None)
            .await?;
        Ok(list.groups)
    }

    pub async fn create_deployment_class(
        &self,
        request: &CreateRuntimeGroup,
    ) -> Result<RuntimeGroup, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/deployment-classes", Some(&body))
            .await
    }

    pub async fn update_deployment_class(
        &self,
        id: RuntimeGroupId,
        request: &UpdateRuntimeGroup,
    ) -> Result<RuntimeGroup, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PATCH,
            &format!("/api/v1/deployment-classes/{id}"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_deployment_class(&self, id: RuntimeGroupId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/deployment-classes/{id}"),
            None,
        )
        .await
    }

    pub async fn list_runtime_groups(&self) -> Result<Vec<RuntimeGroup>, Error> {
        let list: RuntimeGroupList = self
            .send(Method::GET, "/api/v1/runtime-groups", None)
            .await?;
        Ok(list.groups)
    }

    pub async fn create_runtime_group(
        &self,
        request: &CreateRuntimeGroup,
    ) -> Result<RuntimeGroup, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/runtime-groups", Some(&body))
            .await
    }

    pub async fn update_runtime_group(
        &self,
        id: RuntimeGroupId,
        request: &UpdateRuntimeGroup,
    ) -> Result<RuntimeGroup, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PATCH,
            &format!("/api/v1/runtime-groups/{id}"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_runtime_group(&self, id: RuntimeGroupId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/runtime-groups/{id}"),
            None,
        )
        .await
    }

    pub async fn runtime_group_usage(
        &self,
        id: RuntimeGroupId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<UsageSummary, Error> {
        self.send(
            Method::GET,
            &format!(
                "/api/v1/runtime-groups/{id}/usage?from={}&to={}",
                urlencoding(&from.to_rfc3339()),
                urlencoding(&to.to_rfc3339())
            ),
            None,
        )
        .await
    }

    // provision confirmation -----------------------------------------------

    pub async fn pod_confirmation(
        &self,
        id: PodId,
        wait_seconds: Option<u64>,
    ) -> Result<ProvisionConfirmation, Error> {
        let suffix = wait_seconds
            .map(|seconds| format!("?wait_seconds={}", seconds.min(120)))
            .unwrap_or_default();
        self.send(
            Method::GET,
            &format!("/api/v1/pods/{id}/confirmation{suffix}"),
            None,
        )
        .await
    }

    // addresses and forwards ----------------------------------------------

    pub async fn list_ips(&self, query: AddressQuery) -> Result<Vec<IpAddress>, Error> {
        let list: IpAddressList = self
            .send(
                Method::GET,
                &format!("/api/v1/ips{}", address_query(&query)),
                None,
            )
            .await?;
        Ok(list.addresses)
    }

    pub async fn create_ip(&self, request: &CreateIp) -> Result<IpAddress, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/ips", Some(&body)).await
    }

    pub async fn get_ip(&self, id: IpAddressId) -> Result<IpAddress, Error> {
        self.send(Method::GET, &format!("/api/v1/ips/{id}"), None)
            .await
    }

    pub async fn update_ip(&self, id: IpAddressId, request: &UpdateIp) -> Result<IpAddress, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::PATCH, &format!("/api/v1/ips/{id}"), Some(&body))
            .await
    }

    pub async fn delete_ip(&self, id: IpAddressId) -> Result<(), Error> {
        self.send_empty(Method::DELETE, &format!("/api/v1/ips/{id}"), None)
            .await
    }

    pub async fn list_forwards(&self, query: AddressQuery) -> Result<Vec<PortForward>, Error> {
        let list: PortForwardList = self
            .send(
                Method::GET,
                &format!("/api/v1/forwards{}", address_query(&query)),
                None,
            )
            .await?;
        Ok(list.forwards)
    }

    pub async fn list_pod_ips(&self, id: PodId) -> Result<Vec<IpAddress>, Error> {
        let list: IpAddressList = self
            .send(Method::GET, &format!("/api/v1/pods/{id}/ips"), None)
            .await?;
        Ok(list.addresses)
    }

    pub async fn bind_pod_ip(&self, id: PodId, request: &BindIp) -> Result<IpAddress, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, &format!("/api/v1/pods/{id}/ips"), Some(&body))
            .await
    }

    pub async fn unbind_pod_ip(&self, id: PodId, ip_id: IpAddressId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/pods/{id}/ips/{ip_id}"),
            None,
        )
        .await
    }

    pub async fn list_pod_forwards(&self, id: PodId) -> Result<Vec<PortForward>, Error> {
        let list: PortForwardList = self
            .send(Method::GET, &format!("/api/v1/pods/{id}/forwards"), None)
            .await?;
        Ok(list.forwards)
    }

    pub async fn create_pod_forward(
        &self,
        id: PodId,
        request: &CreateForward,
    ) -> Result<PortForward, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{id}/forwards"),
            Some(&body),
        )
        .await
    }

    pub async fn update_pod_forward(
        &self,
        id: PodId,
        forward_id: PortForwardId,
        request: &UpdateForward,
    ) -> Result<PortForward, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PATCH,
            &format!("/api/v1/pods/{id}/forwards/{forward_id}"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_pod_forward(
        &self,
        id: PodId,
        forward_id: PortForwardId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/pods/{id}/forwards/{forward_id}"),
            None,
        )
        .await
    }

    // platform -------------------------------------------------------------

    pub async fn get_platform(&self, id: PodId) -> Result<PlatformSpec, Error> {
        self.send(Method::GET, &format!("/api/v1/pods/{id}/platform"), None)
            .await
    }

    pub async fn put_platform(
        &self,
        id: PodId,
        spec: &PlatformSpec,
    ) -> Result<PlatformSpec, Error> {
        let body = serde_json::to_value(spec)?;
        self.send(
            Method::PUT,
            &format!("/api/v1/pods/{id}/platform"),
            Some(&body),
        )
        .await
    }

    // autonomous updates ---------------------------------------------------

    pub async fn get_update_policy(&self, id: PodId) -> Result<UpdatePolicy, Error> {
        self.send(
            Method::GET,
            &format!("/api/v1/pods/{id}/update-policy"),
            None,
        )
        .await
    }

    pub async fn put_update_policy(
        &self,
        id: PodId,
        policy: &UpdatePolicy,
    ) -> Result<UpdatePolicy, Error> {
        let body = serde_json::to_value(policy)?;
        self.send(
            Method::PUT,
            &format!("/api/v1/pods/{id}/update-policy"),
            Some(&body),
        )
        .await
    }

    pub async fn check_pod_updates(&self, id: PodId) -> Result<Value, Error> {
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{id}/updates/check"),
            Some(&serde_json::json!({})),
        )
        .await
    }

    pub async fn list_revisions(&self, id: PodId) -> Result<Vec<Revision>, Error> {
        let list: RevisionList = self
            .send(Method::GET, &format!("/api/v1/pods/{id}/revisions"), None)
            .await?;
        Ok(list.revisions)
    }

    pub async fn delete_revision(&self, id: PodId, revision_id: uuid::Uuid) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/pods/{id}/revisions/{revision_id}"),
            None,
        )
        .await
    }

    pub async fn pin_revision(
        &self,
        id: PodId,
        revision_id: uuid::Uuid,
        pinned: bool,
    ) -> Result<Revision, Error> {
        let body = serde_json::json!({"pinned": pinned});
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{id}/revisions/{revision_id}/pin"),
            Some(&body),
        )
        .await
    }

    pub async fn get_rollout(&self, id: PodId) -> Result<Option<Rollout>, Error> {
        let status: RolloutStatus = self
            .send(Method::GET, &format!("/api/v1/pods/{id}/rollout"), None)
            .await?;
        Ok(status.rollout)
    }

    // metering -------------------------------------------------------------

    pub async fn get_meter_view(
        &self,
        scope: &str,
        scope_id: Option<uuid::Uuid>,
    ) -> Result<MeterView, Error> {
        let suffix = scope_id
            .map(|id| format!("&scope_id={id}"))
            .unwrap_or_default();
        self.send(
            Method::GET,
            &format!("/api/v1/metering?scope={}{suffix}", urlencoding(scope)),
            None,
        )
        .await
    }

    pub async fn set_meter_view(&self, request: &MeterViewRequest) -> Result<MeterView, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::PUT, "/api/v1/metering", Some(&body))
            .await
    }

    pub async fn pod_usage(
        &self,
        id: PodId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<UsageSummary, Error> {
        self.send(
            Method::GET,
            &format!(
                "/api/v1/pods/{id}/usage?from={}&to={}",
                urlencoding(&from.to_rfc3339()),
                urlencoding(&to.to_rfc3339())
            ),
            None,
        )
        .await
    }

    pub async fn pod_usage_samples(
        &self,
        id: PodId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<UsageSample>, Error> {
        let usage: RawUsage = self
            .send(
                Method::GET,
                &format!(
                    "/api/v1/pods/{id}/usage/samples?from={}&to={}",
                    urlencoding(&from.to_rfc3339()),
                    urlencoding(&to.to_rfc3339())
                ),
                None,
            )
            .await?;
        Ok(usage.samples)
    }

    // jobs and transfer ----------------------------------------------------

    pub async fn list_all_jobs(&self, query: JobQuery) -> Result<Vec<Job>, Error> {
        let mut params = Vec::new();
        if let Some(status) = query.status {
            params.push(format!("status={}", status.as_str()));
        }
        if let Some(limit) = query.limit {
            params.push(format!("limit={limit}"));
        }
        let suffix = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };
        let list: GlobalJobList = self
            .send(Method::GET, &format!("/api/v1/jobs{suffix}"), None)
            .await?;
        Ok(list.jobs)
    }

    pub async fn get_job(&self, id: JobId) -> Result<Job, Error> {
        self.send(Method::GET, &format!("/api/v1/jobs/{id}"), None)
            .await
    }

    pub async fn retry_job(&self, id: JobId) -> Result<Job, Error> {
        self.send(
            Method::POST,
            &format!("/api/v1/jobs/{id}/retry"),
            Some(&serde_json::json!({})),
        )
        .await
    }

    pub async fn transfer_pod(
        &self,
        id: PodId,
        request: &TransferRequest,
    ) -> Result<TransferAccepted, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{id}/transfer"),
            Some(&body),
        )
        .await
    }

    // webhooks -------------------------------------------------------------

    pub async fn list_webhooks(&self) -> Result<Vec<WebhookSubscription>, Error> {
        let list: WebhookList = self.send(Method::GET, "/api/v1/webhooks", None).await?;
        Ok(list.webhooks)
    }

    pub async fn create_webhook(&self, request: &CreateWebhook) -> Result<CreatedWebhook, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/webhooks", Some(&body))
            .await
    }

    pub async fn update_webhook(
        &self,
        id: WebhookId,
        request: &UpdateWebhook,
    ) -> Result<WebhookSubscription, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PATCH,
            &format!("/api/v1/webhooks/{id}"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_webhook(&self, id: WebhookId) -> Result<(), Error> {
        self.send_empty(Method::DELETE, &format!("/api/v1/webhooks/{id}"), None)
            .await
    }

    pub async fn rotate_webhook_secret(&self, id: WebhookId) -> Result<String, Error> {
        let rotated: RotatedWebhookSecret = self
            .send(
                Method::POST,
                &format!("/api/v1/webhooks/{id}/rotate-secret"),
                Some(&serde_json::json!({})),
            )
            .await?;
        Ok(rotated.secret)
    }

    pub async fn list_webhook_deliveries(
        &self,
        id: WebhookId,
    ) -> Result<Vec<WebhookDelivery>, Error> {
        let list: WebhookDeliveryList = self
            .send(
                Method::GET,
                &format!("/api/v1/webhooks/{id}/deliveries"),
                None,
            )
            .await?;
        Ok(list.deliveries)
    }

    // template nests -------------------------------------------------------

    pub async fn import_template_toml(&self, body: &str) -> Result<ImportResult, Error> {
        let response = self
            .http
            .post(format!(
                "{}/api/v1/templates/import?save=true",
                self.base_url
            ))
            .bearer_auth(&self.api_key)
            .header("content-type", "application/toml")
            .body(body.to_string())
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(parse_error(status, &text));
        }
        Ok(serde_json::from_str(&text)?)
    }

    pub async fn export_template_toml(&self, id: TemplateId) -> Result<String, Error> {
        let response = self
            .http
            .get(format!(
                "{}/api/v1/templates/{id}/export?format=toml",
                self.base_url
            ))
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

    pub async fn convert_compose(&self, yaml: &str) -> Result<ComposeConverted, Error> {
        let response = self
            .http
            .post(format!("{}/api/v1/compose/convert", self.base_url))
            .bearer_auth(&self.api_key)
            .header("content-type", "application/yaml")
            .body(yaml.to_string())
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(parse_error(status, &text));
        }
        Ok(serde_json::from_str(&text)?)
    }

    pub async fn import_nest(&self, request: &ImportNest) -> Result<NestImportResult, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/templates/nests", Some(&body))
            .await
    }

    // proxy adapters and remaining fabric ---------------------------------

    pub async fn list_proxy_adapters(&self) -> Result<AdapterList, Error> {
        self.send(Method::GET, "/api/v1/proxy/adapters", None).await
    }

    pub async fn create_proxy_adapter(
        &self,
        request: &CreateAdapter,
    ) -> Result<ProxyAdapter, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/proxy/adapters", Some(&body))
            .await
    }

    pub async fn delete_proxy_adapter(&self, id: ProxyAdapterId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/proxy/adapters/{id}"),
            None,
        )
        .await
    }

    pub async fn delete_vnet(&self, id: VnetId) -> Result<(), Error> {
        self.send_empty(Method::DELETE, &format!("/api/v1/vnets/{id}"), None)
            .await
    }

    pub async fn detach_vnet(&self, id: VnetId, pod_id: PodId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/vnets/{id}/attachments/{pod_id}"),
            None,
        )
        .await
    }

    pub async fn delete_interconnect(&self, id: uuid::Uuid) -> Result<(), Error> {
        self.send_empty(Method::DELETE, &format!("/api/v1/interconnects/{id}"), None)
            .await
    }

    pub async fn delete_listener(&self, id: uuid::Uuid) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/proxy/listeners/{id}"),
            None,
        )
        .await
    }

    pub async fn list_listener_routes(
        &self,
        id: uuid::Uuid,
    ) -> Result<Vec<ProxyListenerRoute>, Error> {
        let list: ListenerRouteList = self
            .send(
                Method::GET,
                &format!("/api/v1/proxy/listeners/{id}/routes"),
                None,
            )
            .await?;
        Ok(list.routes)
    }

    pub async fn delete_listener_route(
        &self,
        id: uuid::Uuid,
        route_id: uuid::Uuid,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/proxy/listeners/{id}/routes/{route_id}"),
            None,
        )
        .await
    }

    // files and limits -----------------------------------------------------

    pub async fn download_file(&self, id: PodId, path: &str) -> Result<Vec<u8>, Error> {
        self.send_bytes(&format!(
            "/api/v1/pods/{id}/files/download?path={}",
            urlencoding(path)
        ))
        .await
    }

    pub async fn upload_file(
        &self,
        id: PodId,
        path: &str,
        bytes: Vec<u8>,
    ) -> Result<FileEntry, Error> {
        let response = self
            .http
            .post(format!("{}/api/v1/pods/{id}/files/upload", self.base_url))
            .bearer_auth(&self.api_key)
            .header("content-type", "application/octet-stream")
            .query(&[("path", path)])
            .body(bytes)
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(parse_error(status, &text));
        }
        Ok(serde_json::from_str(&text)?)
    }

    pub async fn update_limits(&self, id: PodId, limits: &Limits) -> Result<Pod, Error> {
        let body = serde_json::json!({"limits": limits});
        self.send(
            Method::PATCH,
            &format!("/api/v1/pods/{id}/limits"),
            Some(&body),
        )
        .await
    }

    pub async fn deploy_compose(
        &self,
        request: &DeployCompose,
    ) -> Result<ComposeDeployResult, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/compose/deploy", Some(&body))
            .await
    }

    // storage ---------------------------------------------------------------

    pub async fn list_storage_backends(
        &self,
        tenant_id: Option<TenantId>,
    ) -> Result<Vec<StorageBackend>, Error> {
        let suffix = tenant_id
            .map(|id| format!("?tenant_id={id}"))
            .unwrap_or_default();
        let list: BackendList = self
            .send(
                Method::GET,
                &format!("/api/v1/storage/backends{suffix}"),
                None,
            )
            .await?;
        Ok(list.backends)
    }

    pub async fn create_storage_backend(
        &self,
        request: &CreateStorageBackend,
    ) -> Result<StorageBackend, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/storage/backends", Some(&body))
            .await
    }

    pub async fn get_storage_backend(&self, id: StorageBackendId) -> Result<StorageBackend, Error> {
        self.send(Method::GET, &format!("/api/v1/storage/backends/{id}"), None)
            .await
    }

    pub async fn update_storage_backend(
        &self,
        id: StorageBackendId,
        request: &UpdateStorageBackend,
    ) -> Result<StorageBackend, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PATCH,
            &format!("/api/v1/storage/backends/{id}"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_storage_backend(&self, id: StorageBackendId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/storage/backends/{id}"),
            None,
        )
        .await
    }

    pub async fn list_volumes(
        &self,
        resource_group_id: Option<ResourceGroupId>,
    ) -> Result<Vec<Volume>, Error> {
        let suffix = resource_group_id
            .map(|id| format!("?resource_group_id={id}"))
            .unwrap_or_default();
        let list: VolumeList = self
            .send(
                Method::GET,
                &format!("/api/v1/storage/volumes{suffix}"),
                None,
            )
            .await?;
        Ok(list.volumes)
    }

    pub async fn create_volume(&self, request: &CreateVolume) -> Result<Volume, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/storage/volumes", Some(&body))
            .await
    }

    pub async fn get_volume(&self, id: VolumeId) -> Result<Volume, Error> {
        self.send(Method::GET, &format!("/api/v1/storage/volumes/{id}"), None)
            .await
    }

    pub async fn update_volume(
        &self,
        id: VolumeId,
        request: &UpdateVolume,
    ) -> Result<Volume, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PATCH,
            &format!("/api/v1/storage/volumes/{id}"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_volume(&self, id: VolumeId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/storage/volumes/{id}"),
            None,
        )
        .await
    }

    pub async fn list_pod_volumes(&self, pod_id: PodId) -> Result<Vec<PodVolumeMountView>, Error> {
        let list: MountList = self
            .send(Method::GET, &format!("/api/v1/pods/{pod_id}/volumes"), None)
            .await?;
        Ok(list.mounts)
    }

    pub async fn list_storage_gateway_tokens(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<StorageGatewayToken>, Error> {
        let list: StorageGatewayTokenList = self
            .send(
                Method::GET,
                &format!("/api/v1/storage/gateway-tokens?tenant_id={tenant_id}"),
                None,
            )
            .await?;
        Ok(list.tokens)
    }

    pub async fn create_storage_gateway_token(
        &self,
        request: &CreateStorageGatewayToken,
    ) -> Result<CreatedStorageGatewayToken, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/storage/gateway-tokens", Some(&body))
            .await
    }

    pub async fn revoke_storage_gateway_token(
        &self,
        id: StorageGatewayTokenId,
        tenant_id: TenantId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/storage/gateway-tokens/{id}?tenant_id={tenant_id}"),
            None,
        )
        .await
    }

    pub async fn report_volume_usage(
        &self,
        id: VolumeId,
        used_bytes: u64,
    ) -> Result<Volume, Error> {
        let body = serde_json::json!({ "used_bytes": used_bytes });
        self.send(
            Method::POST,
            &format!("/api/v1/storage/volumes/{id}/usage"),
            Some(&body),
        )
        .await
    }

    pub async fn mount_volume(
        &self,
        pod_id: PodId,
        request: &MountVolume,
    ) -> Result<PodVolumeMount, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{pod_id}/volumes"),
            Some(&body),
        )
        .await
    }

    pub async fn unmount_volume(
        &self,
        pod_id: PodId,
        mount_id: PodVolumeMountId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/pods/{pod_id}/volumes/{mount_id}"),
            None,
        )
        .await
    }

    pub async fn list_external_databases(
        &self,
        tenant_id: Option<TenantId>,
    ) -> Result<Vec<ExternalDatabase>, Error> {
        let suffix = tenant_id
            .map(|id| format!("?tenant_id={id}"))
            .unwrap_or_default();
        let list: ExternalDatabaseList = self
            .send(
                Method::GET,
                &format!("/api/v1/external-databases{suffix}"),
                None,
            )
            .await?;
        Ok(list.databases)
    }

    pub async fn create_external_database(
        &self,
        request: &CreateExternalDatabase,
    ) -> Result<ExternalDatabase, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/external-databases", Some(&body))
            .await
    }

    pub async fn get_external_database(
        &self,
        id: ExternalDatabaseId,
    ) -> Result<ExternalDatabase, Error> {
        self.send(
            Method::GET,
            &format!("/api/v1/external-databases/{id}"),
            None,
        )
        .await
    }

    pub async fn update_external_database(
        &self,
        id: ExternalDatabaseId,
        request: &UpdateExternalDatabase,
    ) -> Result<ExternalDatabase, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PATCH,
            &format!("/api/v1/external-databases/{id}"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_external_database(&self, id: ExternalDatabaseId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/external-databases/{id}"),
            None,
        )
        .await
    }

    pub async fn test_external_database(
        &self,
        id: ExternalDatabaseId,
    ) -> Result<DatabaseTest, Error> {
        self.send(
            Method::POST,
            &format!("/api/v1/external-databases/{id}/test"),
            Some(&serde_json::json!({})),
        )
        .await
    }

    pub async fn list_pod_external_databases(
        &self,
        pod_id: PodId,
    ) -> Result<Vec<PodExternalDatabaseView>, Error> {
        let list: ExternalDatabaseBindingList = self
            .send(
                Method::GET,
                &format!("/api/v1/pods/{pod_id}/external-databases"),
                None,
            )
            .await?;
        Ok(list.bindings)
    }

    pub async fn bind_pod_external_database(
        &self,
        pod_id: PodId,
        request: &BindExternalDatabase,
    ) -> Result<PodExternalDatabase, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{pod_id}/external-databases"),
            Some(&body),
        )
        .await
    }

    pub async fn unbind_pod_external_database(
        &self,
        pod_id: PodId,
        binding_id: PodExternalDatabaseId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/pods/{pod_id}/external-databases/{binding_id}"),
            None,
        )
        .await
    }

    // metrics storage -------------------------------------------------------

    pub async fn get_metrics_storage_policy(
        &self,
        tenant_id: Option<TenantId>,
    ) -> Result<MetricsStoragePolicy, Error> {
        let suffix = tenant_id
            .map(|id| format!("?tenant_id={id}"))
            .unwrap_or_default();
        self.send(
            Method::GET,
            &format!("/api/v1/metrics/storage-policy{suffix}"),
            None,
        )
        .await
    }

    pub async fn put_metrics_storage_policy(
        &self,
        tenant_id: Option<TenantId>,
        request: &UpsertMetricsStoragePolicy,
    ) -> Result<MetricsStoragePolicy, Error> {
        let suffix = tenant_id
            .map(|id| format!("?tenant_id={id}"))
            .unwrap_or_default();
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PUT,
            &format!("/api/v1/metrics/storage-policy{suffix}"),
            Some(&body),
        )
        .await
    }

    pub async fn list_metrics_archives(
        &self,
        tenant_id: Option<TenantId>,
    ) -> Result<Vec<MetricsArchive>, Error> {
        let suffix = tenant_id
            .map(|id| format!("?tenant_id={id}"))
            .unwrap_or_default();
        let list: MetricsArchiveList = self
            .send(
                Method::GET,
                &format!("/api/v1/metrics/archive{suffix}"),
                None,
            )
            .await?;
        Ok(list.archives)
    }

    pub async fn archive_metrics(
        &self,
        tenant_id: Option<TenantId>,
    ) -> Result<MetricsArchiveReport, Error> {
        let suffix = tenant_id
            .map(|id| format!("?tenant_id={id}"))
            .unwrap_or_default();
        self.send(
            Method::POST,
            &format!("/api/v1/metrics/archive-now{suffix}"),
            Some(&serde_json::json!({})),
        )
        .await
    }

    // git ------------------------------------------------------------------

    pub async fn list_git_credentials(&self) -> Result<Vec<GitCredential>, Error> {
        self.send(Method::GET, "/api/v1/git/credentials", None)
            .await
    }

    pub async fn create_git_credential(
        &self,
        request: &CreateGitCredential,
    ) -> Result<GitCredential, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/git/credentials", Some(&body))
            .await
    }

    pub async fn get_git_credential(&self, id: GitCredentialId) -> Result<GitCredential, Error> {
        self.send(Method::GET, &format!("/api/v1/git/credentials/{id}"), None)
            .await
    }

    pub async fn delete_git_credential(&self, id: GitCredentialId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/git/credentials/{id}"),
            None,
        )
        .await
    }

    pub async fn list_git_sources(&self) -> Result<Vec<GitSource>, Error> {
        self.send(Method::GET, "/api/v1/git/sources", None).await
    }

    pub async fn create_git_source(
        &self,
        request: &CreateGitSource,
    ) -> Result<CreatedGitSource, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/git/sources", Some(&body))
            .await
    }

    pub async fn get_git_source(&self, id: GitSourceId) -> Result<GitSource, Error> {
        self.send(Method::GET, &format!("/api/v1/git/sources/{id}"), None)
            .await
    }

    pub async fn update_git_source(
        &self,
        id: GitSourceId,
        request: &UpdateGitSource,
    ) -> Result<GitSource, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PATCH,
            &format!("/api/v1/git/sources/{id}"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_git_source(&self, id: GitSourceId) -> Result<(), Error> {
        self.send_empty(Method::DELETE, &format!("/api/v1/git/sources/{id}"), None)
            .await
    }

    pub async fn trigger_git_build(
        &self,
        id: GitSourceId,
        request: &TriggerGitBuild,
    ) -> Result<GitBuild, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/git/sources/{id}/build"),
            Some(&body),
        )
        .await
    }

    pub async fn list_git_builds(&self) -> Result<Vec<GitBuild>, Error> {
        let list: BuildList = self.send(Method::GET, "/api/v1/git/builds", None).await?;
        Ok(list.builds)
    }

    pub async fn get_git_build(&self, id: BuildId) -> Result<GitBuild, Error> {
        self.send(Method::GET, &format!("/api/v1/git/builds/{id}"), None)
            .await
    }

    // images ---------------------------------------------------------------

    pub async fn list_images(&self) -> Result<Vec<Image>, Error> {
        let list: ImageList = self.send(Method::GET, "/api/v1/images", None).await?;
        Ok(list.images)
    }

    pub async fn get_image(&self, id: ImageId) -> Result<Image, Error> {
        self.send(Method::GET, &format!("/api/v1/images/{id}"), None)
            .await
    }

    pub async fn image_copies(&self, id: ImageId) -> Result<Vec<ImageCopy>, Error> {
        self.send(Method::GET, &format!("/api/v1/images/{id}/copies"), None)
            .await
    }

    pub async fn image_pulls(&self, id: ImageId) -> Result<Vec<ImagePull>, Error> {
        self.send(Method::GET, &format!("/api/v1/images/{id}/pulls"), None)
            .await
    }

    pub async fn image_usage(&self) -> Result<Vec<serde_json::Value>, Error> {
        self.send(Method::GET, "/api/v1/images/usage", None).await
    }

    pub async fn all_image_pulls(&self) -> Result<Vec<ImagePull>, Error> {
        self.send(Method::GET, "/api/v1/images/pulls", None).await
    }

    pub async fn pull_image(&self, request: &PullImage) -> Result<Image, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/images/pull", Some(&body))
            .await
    }

    pub async fn upload_image(&self, request: &UploadImage) -> Result<Image, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/images/upload", Some(&body))
            .await
    }

    pub async fn delete_image(&self, id: ImageId) -> Result<(), Error> {
        self.send_empty(Method::DELETE, &format!("/api/v1/images/{id}"), None)
            .await
    }

    // pod port bindings ------------------------------------------------

    pub async fn list_pod_ports(&self, id: PodId) -> Result<Vec<PodPort>, Error> {
        let list: PodPortList = self
            .send(Method::GET, &format!("/api/v1/pods/{id}/ports"), None)
            .await?;
        Ok(list.ports)
    }

    pub async fn create_pod_port(
        &self,
        id: PodId,
        request: &CreatePodPort,
    ) -> Result<PodPort, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/pods/{id}/ports"),
            Some(&body),
        )
        .await
    }

    pub async fn update_pod_port(
        &self,
        id: PodId,
        port_id: PodPortId,
        request: &UpdatePodPort,
    ) -> Result<PodPort, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PATCH,
            &format!("/api/v1/pods/{id}/ports/{port_id}"),
            Some(&body),
        )
        .await
    }

    pub async fn pod_port_history(&self, id: PodId) -> Result<Vec<PodPortEvent>, Error> {
        let list: PortHistory = self
            .send(
                Method::GET,
                &format!("/api/v1/pods/{id}/ports/history"),
                None,
            )
            .await?;
        Ok(list.events)
    }

    pub async fn delete_pod_port(&self, id: PodId, port_id: PodPortId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/pods/{id}/ports/{port_id}"),
            None,
        )
        .await
    }

    // tenants and resource groups -----------------------------------------

    pub async fn list_tenants(&self) -> Result<Vec<Tenant>, Error> {
        self.send(Method::GET, "/api/v1/tenants", None).await
    }

    pub async fn create_tenant(&self, request: &CreateTenant) -> Result<Tenant, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/tenants", Some(&body))
            .await
    }

    pub async fn get_tenant(&self, id: TenantId) -> Result<Tenant, Error> {
        self.send(Method::GET, &format!("/api/v1/tenants/{id}"), None)
            .await
    }

    pub async fn update_tenant(
        &self,
        id: TenantId,
        request: &UpdateTenant,
    ) -> Result<Tenant, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::PATCH, &format!("/api/v1/tenants/{id}"), Some(&body))
            .await
    }

    pub async fn list_tenant_access(&self, tenant_id: TenantId) -> Result<Vec<uuid::Uuid>, Error> {
        let list: TenantAccessList = self
            .send(
                Method::GET,
                &format!("/api/v1/tenants/{tenant_id}/access"),
                None,
            )
            .await?;
        Ok(list.api_key_ids)
    }

    pub async fn grant_tenant_access(
        &self,
        tenant_id: TenantId,
        request: &GrantTenantAccess,
    ) -> Result<Vec<uuid::Uuid>, Error> {
        let body = serde_json::to_value(request)?;
        let list: TenantAccessList = self
            .send(
                Method::POST,
                &format!("/api/v1/tenants/{tenant_id}/access"),
                Some(&body),
            )
            .await?;
        Ok(list.api_key_ids)
    }

    pub async fn revoke_tenant_access(
        &self,
        tenant_id: TenantId,
        key_id: uuid::Uuid,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/tenants/{tenant_id}/access/{key_id}"),
            None,
        )
        .await
    }

    pub async fn list_tenant_resource_groups(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<ResourceGroup>, Error> {
        self.send(
            Method::GET,
            &format!("/api/v1/tenants/{tenant_id}/resource-groups"),
            None,
        )
        .await
    }

    pub async fn create_resource_group(
        &self,
        tenant_id: TenantId,
        request: &CreateResourceGroup,
    ) -> Result<ResourceGroup, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/tenants/{tenant_id}/resource-groups"),
            Some(&body),
        )
        .await
    }

    pub async fn list_resource_groups(&self) -> Result<Vec<ResourceGroup>, Error> {
        self.send(Method::GET, "/api/v1/resource-groups", None)
            .await
    }

    pub async fn get_resource_group(&self, id: ResourceGroupId) -> Result<ResourceGroup, Error> {
        self.send(Method::GET, &format!("/api/v1/resource-groups/{id}"), None)
            .await
    }

    pub async fn update_resource_group(
        &self,
        id: ResourceGroupId,
        request: &UpdateResourceGroup,
    ) -> Result<ResourceGroup, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::PATCH,
            &format!("/api/v1/resource-groups/{id}"),
            Some(&body),
        )
        .await
    }

    pub async fn delete_resource_group(&self, id: ResourceGroupId) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/resource-groups/{id}"),
            None,
        )
        .await
    }

    pub async fn get_reference(&self, reference_id: &str) -> Result<ReferenceAggregate, Error> {
        self.send(
            Method::GET,
            &format!("/api/v1/references/{}", urlencoding(reference_id)),
            None,
        )
        .await
    }

    // transfers ------------------------------------------------------------

    pub async fn create_transfer_key(
        &self,
        request: &CreateTransferKey,
    ) -> Result<CreatedTransferKey, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/transfer-keys", Some(&body))
            .await
    }

    pub async fn list_transfer_keys(&self, tenant_id: TenantId) -> Result<Vec<TransferKey>, Error> {
        self.send(
            Method::GET,
            &format!("/api/v1/transfer-keys?tenant_id={tenant_id}"),
            None,
        )
        .await
    }

    pub async fn revoke_transfer_key(
        &self,
        id: TransferKeyId,
        tenant_id: TenantId,
    ) -> Result<(), Error> {
        self.send_empty(
            Method::DELETE,
            &format!("/api/v1/transfer-keys/{id}?tenant_id={tenant_id}"),
            None,
        )
        .await
    }

    pub async fn start_transfer(
        &self,
        resource_group_id: ResourceGroupId,
        request: &StartTransfer,
    ) -> Result<TenantTransfer, Error> {
        let body = serde_json::to_value(request)?;
        self.send(
            Method::POST,
            &format!("/api/v1/resource-groups/{resource_group_id}/transfer"),
            Some(&body),
        )
        .await
    }

    pub async fn list_transfers(
        &self,
        resource_group_id: ResourceGroupId,
    ) -> Result<Vec<TenantTransfer>, Error> {
        self.send(
            Method::GET,
            &format!("/api/v1/resource-groups/{resource_group_id}/transfers"),
            None,
        )
        .await
    }

    pub async fn get_transfer(&self, id: TenantTransferId) -> Result<TenantTransfer, Error> {
        self.send(Method::GET, &format!("/api/v1/transfers/{id}"), None)
            .await
    }

    pub async fn cancel_transfer(&self, id: TenantTransferId) -> Result<TenantTransfer, Error> {
        self.send(
            Method::POST,
            &format!("/api/v1/transfers/{id}/cancel"),
            Some(&serde_json::json!({})),
        )
        .await
    }

    pub async fn dismiss_transfer(&self, id: TenantTransferId) -> Result<TenantTransfer, Error> {
        self.send(
            Method::POST,
            &format!("/api/v1/transfers/{id}/dismiss"),
            Some(&serde_json::json!({})),
        )
        .await
    }

    // usage and token profiles --------------------------------------------

    pub async fn usage(
        &self,
        scope: &str,
        scope_id: uuid::Uuid,
        window: &str,
        unit: &str,
        include_exact: bool,
    ) -> Result<UsageValue, Error> {
        self.send(
            Method::GET,
            &format!(
                "/api/v1/usage?scope={}&scope_id={scope_id}&window={}&unit={unit}&include_exact={include_exact}",
                urlencoding(scope),
                urlencoding(window)
            ),
            None,
        )
        .await
    }

    pub async fn list_metering_profiles(&self) -> Result<Vec<MeteringProfile>, Error> {
        self.send(Method::GET, "/api/v1/metering/profiles", None)
            .await
    }

    pub async fn create_metering_profile(
        &self,
        request: &CreateMeteringProfile,
    ) -> Result<MeteringProfile, Error> {
        let body = serde_json::to_value(request)?;
        self.send(Method::POST, "/api/v1/metering/profiles", Some(&body))
            .await
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

fn address_query(query: &AddressQuery) -> String {
    let mut params = Vec::new();
    if let Some(node_id) = query.node_id {
        params.push(format!("node_id={node_id}"));
    }
    if let Some(pod_id) = query.pod_id {
        params.push(format!("pod_id={pod_id}"));
    }
    if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_and_runtime_group_placement_serialize_exclusively() {
        let by_node =
            serde_json::to_value(CreatePod::new(NodeId::new(), TemplateId::new(), "survival"))
                .unwrap();
        assert!(by_node.get("node_id").is_some());
        assert!(by_node.get("runtime_group_id").is_none());

        let by_group = serde_json::to_value(CreatePod::in_runtime_group(
            RuntimeGroupId::new(),
            TemplateId::new(),
            "survival",
        ))
        .unwrap();
        assert!(by_group.get("node_id").is_none());
        assert!(by_group.get("runtime_group_id").is_some());
    }

    #[test]
    fn metering_unit_uses_the_wire_name() {
        let view = MeterViewRequest {
            scope: "default".into(),
            scope_id: None,
            unit: MeterUnit::Tokens,
        };
        assert_eq!(
            serde_json::to_value(&view).unwrap()["unit"],
            serde_json::Value::String("tokens".into())
        );
        assert!(serde_json::to_value(&view)
            .unwrap()
            .get("scope_id")
            .is_none());
    }

    #[test]
    fn ip_creation_uses_snake_case_kinds() {
        let value = serde_json::to_value(CreateIp::new("185.16.3.19", IpKind::Floating)).unwrap();
        assert_eq!(value["kind"], serde_json::Value::String("floating".into()));
        assert_eq!(value["ip"], serde_json::Value::String("185.16.3.19".into()));
    }

    #[test]
    fn clearing_a_webhook_description_sends_null() {
        let update = UpdateWebhook {
            description: Some(None),
            ..Default::default()
        };
        assert!(serde_json::to_value(&update).unwrap()["description"].is_null());
    }

    #[test]
    fn nest_import_keeps_an_explicit_save_false() {
        let mut request = ImportNest::new(serde_json::json!({"eggs": []}));
        request.save = false;
        assert_eq!(serde_json::to_value(&request).unwrap()["save"], false);
    }
}
