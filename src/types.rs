//! Standalone API model types.
//!
//! These mirror the public control-plane schema but intentionally live inside the
//! client crate so that the client does not depend on any other Ignite Containers
//! crate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub uuid::Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }

            #[must_use]
            pub fn from_uuid(uuid: uuid::Uuid) -> Self {
                Self(uuid)
            }

            #[must_use]
            pub fn as_uuid(&self) -> uuid::Uuid {
                self.0
            }

            #[must_use]
            pub fn parse(value: &str) -> Option<Self> {
                uuid::Uuid::parse_str(value).ok().map(Self)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                uuid::Uuid::parse_str(value).map(Self)
            }
        }

        impl From<uuid::Uuid> for $name {
            fn from(value: uuid::Uuid) -> Self {
                Self(value)
            }
        }
    };
}

id_type!(ApiKeyId);
id_type!(NodeId);
id_type!(AllocationId);
id_type!(TemplateId);
id_type!(TemplateVersionId);
id_type!(PodId);
id_type!(JobId);
id_type!(DatabaseHostId);
id_type!(PodDatabaseId);
id_type!(BackupId);
id_type!(ScheduleId);
id_type!(EmailId);
id_type!(SftpCredentialId);
id_type!(VnetId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Scope {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "apikey:read")]
    ApiKeyRead,
    #[serde(rename = "apikey:write")]
    ApiKeyWrite,
    #[serde(rename = "audit:read")]
    AuditRead,
    #[serde(rename = "node:read")]
    NodeRead,
    #[serde(rename = "node:write")]
    NodeWrite,
    #[serde(rename = "allocation:read")]
    AllocationRead,
    #[serde(rename = "allocation:write")]
    AllocationWrite,
    #[serde(rename = "template:read")]
    TemplateRead,
    #[serde(rename = "template:write")]
    TemplateWrite,
    #[serde(rename = "pod:read")]
    PodRead,
    #[serde(rename = "pod:write")]
    PodWrite,
    #[serde(rename = "pod:power")]
    PodPower,
    #[serde(rename = "pod:console")]
    PodConsole,
    #[serde(rename = "pod:files")]
    PodFiles,
    #[serde(rename = "pod:backup")]
    PodBackup,
    #[serde(rename = "database:read")]
    DatabaseRead,
    #[serde(rename = "database:write")]
    DatabaseWrite,
    #[serde(rename = "pod:database")]
    PodDatabase,
    #[serde(rename = "pod:sftp")]
    PodSftp,
    #[serde(rename = "pod:secrets")]
    PodSecrets,
    #[serde(rename = "notification:send")]
    NotificationSend,
    #[serde(rename = "notification:read")]
    NotificationRead,
    #[serde(rename = "proxy:read")]
    ProxyRead,
    #[serde(rename = "proxy:write")]
    ProxyWrite,
    #[serde(rename = "webhook:read")]
    WebhookRead,
    #[serde(rename = "webhook:write")]
    WebhookWrite,
    #[serde(rename = "vnet:read")]
    VnetRead,
    #[serde(rename = "vnet:write")]
    VnetWrite,
    #[serde(rename = "ip:read")]
    IpRead,
    #[serde(rename = "ip:write")]
    IpWrite,
    #[serde(rename = "tenant:read")]
    TenantRead,
    #[serde(rename = "tenant:write")]
    TenantWrite,
    #[serde(rename = "tenant:create")]
    TenantCreate,
    #[serde(rename = "tenant:transfer")]
    TenantTransfer,
    #[serde(rename = "resource-group:read")]
    ResourceGroupRead,
    #[serde(rename = "resource-group:write")]
    ResourceGroupWrite,
    #[serde(rename = "metering:read")]
    MeteringRead,
    #[serde(rename = "metering:write")]
    MeteringWrite,
    #[serde(rename = "git:read")]
    GitRead,
    #[serde(rename = "git:write")]
    GitWrite,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScopeSet(pub BTreeSet<Scope>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Limits {
    pub memory_mb: u64,
    pub disk_mb: u64,
    pub cpu_millis: u64,
    pub swap_mb: u64,
    pub io_weight: u16,
    pub pids_limit: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpuset: Option<String>,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            memory_mb: 1024,
            disk_mb: 10_240,
            cpu_millis: 1000,
            swap_mb: 0,
            io_weight: 500,
            pids_limit: 512,
            cpuset: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCapacity {
    pub memory_mb: u64,
    pub disk_mb: u64,
    pub cpu_millis: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeDriver {
    Docker,
    Kubernetes,
}

impl NodeDriver {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            NodeDriver::Docker => "docker",
            NodeDriver::Kubernetes => "kubernetes",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Pending,
    Online,
    Degraded,
    Offline,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub name: String,
    pub fqdn: String,
    pub endpoint: String,
    pub driver: NodeDriver,
    pub status: NodeStatus,
    #[serde(default)]
    pub maintenance: bool,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    pub capacity: NodeCapacity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_version: Option<String>,
    #[serde(default)]
    pub host_updates_supported: bool,
    #[serde(default)]
    pub updates_available: u32,
    #[serde(default)]
    pub reboot_required: bool,
    #[serde(default)]
    pub host_updates_auto_reboot: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_update_check: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_update_apply: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_update_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_group_id: Option<RuntimeGroupId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dedicated_tenant_id: Option<TenantId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PortProtocol {
    Tcp,
    Udp,
    Both,
}

impl PortProtocol {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            PortProtocol::Tcp => "tcp",
            PortProtocol::Udp => "udp",
            PortProtocol::Both => "both",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    pub id: AllocationId,
    pub node_id: NodeId,
    pub ip: String,
    pub port: u16,
    pub protocol: PortProtocol,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pod_id: Option<PodId>,
    #[serde(default)]
    pub primary: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PodState {
    Installing,
    Offline,
    Starting,
    Running,
    Stopping,
    Suspended,
    Pending,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pod {
    pub id: PodId,
    pub node_id: NodeId,
    pub template_id: TemplateId,
    pub template_version: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_key_id: Option<ApiKeyId>,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    pub image: String,
    pub startup: String,
    #[serde(default, skip_serializing)]
    pub environment: BTreeMap<String, String>,
    pub limits: Limits,
    /// Per-pod stop override. `None` inherits the template runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<StopConfig>,
    pub state: PodState,
    #[serde(default)]
    pub desired_running: bool,
    #[serde(default)]
    pub suspended: bool,
    #[serde(default)]
    pub generation: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_allocation_id: Option<AllocationId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodDetail {
    #[serde(flatten)]
    pub pod: Pod,
    pub allocations: Vec<Allocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmation: Option<ProvisionConfirmation>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiKeyLimits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_pods: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_memory_mb: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_disk_mb: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cpu_millis: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: ApiKeyId,
    pub name: String,
    pub prefix: String,
    pub scopes: ScopeSet,
    #[serde(default)]
    pub pod_ids: Vec<PodId>,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_minute: Option<u32>,
    #[serde(default)]
    pub limits: ApiKeyLimits,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHost {
    pub id: DatabaseHostId,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub admin_user: String,
    pub max_databases: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodDatabase {
    pub id: PodDatabaseId,
    pub pod_id: PodId,
    pub host_id: DatabaseHostId,
    pub database_name: String,
    pub username: String,
    pub remote: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupState {
    Creating,
    Completed,
    Failed,
    Restoring,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub id: BackupId,
    pub pod_id: PodId,
    pub name: String,
    pub bytes: u64,
    pub checksum: String,
    pub state: BackupState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerSignal {
    Start,
    Stop,
    Restart,
    Kill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    Create,
    Delete,
    Start,
    Stop,
    Restart,
    Kill,
    Command,
    Reinstall,
    Sync,
    UpdateTemplate,
    Transfer,
    Rollout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl JobStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            JobStatus::Queued => "queued",
            JobStatus::Running => "running",
            JobStatus::Succeeded => "succeeded",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub node_id: NodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pod_id: Option<PodId>,
    pub kind: JobKind,
    #[serde(default)]
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub attempts: u32,
    pub max_attempts: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleActionKind {
    Power,
    Command,
    Backup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleAction {
    #[serde(rename = "type")]
    pub kind: ScheduleActionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal: Option<PowerSignal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: ScheduleId,
    pub pod_id: PodId,
    pub name: String,
    pub cron: String,
    pub actions: Vec<ScheduleAction>,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpCredential {
    pub id: SftpCredentialId,
    pub pod_id: PodId,
    pub username: String,
    pub expires_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailStatus {
    Queued,
    Sending,
    Sent,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub id: EmailId,
    pub recipients: Vec<String>,
    pub subject: String,
    pub body: String,
    pub status: EmailStatus,
    pub attempts: u32,
    pub max_attempts: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Create,
    Update,
    Delete,
    Power,
    Command,
    Import,
    Revoke,
    Login,
    Read,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: uuid::Uuid,
    pub api_key_id: Option<ApiKeyId>,
    pub action: AuditAction,
    pub resource: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_ip_hash: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyTls {
    Auto,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vnet {
    pub id: VnetId,
    pub name: String,
    pub cidr: String,
    pub gateway: String,
    pub scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
    #[serde(default)]
    pub internal: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VnetAttachment {
    pub id: uuid::Uuid,
    pub vnet_id: VnetId,
    pub pod_id: PodId,
    pub ip: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interconnect {
    pub id: uuid::Uuid,
    pub name: String,
    pub vnet_id: VnetId,
    pub pod_ids: Vec<PodId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyListener {
    pub id: uuid::Uuid,
    pub name: String,
    pub node_id: NodeId,
    pub bind_ip: String,
    pub port: u16,
    pub protocol: String,
    pub adapter: String,
    pub real_ip: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub velocity_secret: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyListenerRoute {
    pub id: uuid::Uuid,
    pub listener_id: uuid::Uuid,
    pub match_kind: String,
    pub match_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pod_id: Option<PodId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_port: Option<u16>,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default)]
    pub offline_motd: String,
    #[serde(default)]
    pub queue_timeout_seconds: u32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRoute {
    pub id: uuid::Uuid,
    pub domain: String,
    pub upstream: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pod_id: Option<PodId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_port: Option<u16>,
    pub tls: ProxyTls,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyCertificateStatus {
    Valid,
    Expiring,
    Expired,
    NotYetValid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyCertificate {
    pub id: uuid::Uuid,
    pub domain: String,
    pub certificate_pem: String,
    pub subject: String,
    pub issuer: String,
    pub san: Vec<String>,
    pub serial: String,
    pub fingerprint_sha256: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub chain_length: u32,
    pub days_remaining: i64,
    pub status: ProxyCertificateStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigParser {
    Properties,
    Yaml,
    Json,
    Xml,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigFileSpec {
    pub path: String,
    pub parser: ConfigParser,
    #[serde(default)]
    pub replacements: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub default: String,
    #[serde(default)]
    pub user_viewable: bool,
    #[serde(default)]
    pub user_editable: bool,
    #[serde(default)]
    pub rules: String,
    #[serde(default)]
    pub field_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopKind {
    Command,
    Signal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StopConfig {
    pub kind: StopKind,
    pub value: String,
    #[serde(default = "default_stop_timeout")]
    pub timeout_seconds: u32,
}

fn default_stop_timeout() -> u32 {
    30
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionKind {
    LogLine,
    PortOpen,
    Immediate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartupDetection {
    pub kind: DetectionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default = "default_startup_timeout")]
    pub timeout_seconds: u32,
}

fn default_startup_timeout() -> u32 {
    600
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallSpec {
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(default = "default_install_timeout")]
    pub timeout_seconds: u32,
}

fn default_install_timeout() -> u32 {
    1800
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellMode {
    Auto,
    Never,
    Always,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Runtime {
    pub images: BTreeMap<String, String>,
    pub startup: String,
    pub stop: StopConfig,
    pub detection: StartupDetection,
    pub install: InstallSpec,
    #[serde(default = "default_shell_mode")]
    pub shell: ShellMode,
}

fn default_shell_mode() -> ShellMode {
    ShellMode::Auto
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortPolicy {
    #[serde(default = "default_primary_port_env")]
    pub primary_env: String,
    #[serde(default = "default_protocol")]
    pub default_protocol: PortProtocol,
    #[serde(default)]
    pub max_additional: u8,
}

fn default_primary_port_env() -> String {
    "SERVER_PORT".to_string()
}

fn default_protocol() -> PortProtocol {
    PortProtocol::Tcp
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateOrigin {
    Egg,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateSource {
    pub origin: TemplateOrigin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub egg_author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub egg_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imported_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateMeta {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub features: Vec<String>,
    pub source: TemplateSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Template {
    pub schema: String,
    pub meta: TemplateMeta,
    pub runtime: Runtime,
    #[serde(default)]
    pub variables: Vec<TemplateVariable>,
    #[serde(default)]
    pub config_files: Vec<ConfigFileSpec>,
    #[serde(default)]
    pub file_denylist: Vec<String>,
    pub ports: PortPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Stable,
    Beta,
    Dev,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoUpdate {
    pub enabled: bool,
    pub channel: Channel,
}

impl Default for AutoUpdate {
    fn default() -> Self {
        Self {
            enabled: false,
            channel: Channel::Stable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVersionRecord {
    pub id: TemplateVersionId,
    pub template_id: TemplateId,
    pub version: String,
    pub channel: Channel,
    pub template: Template,
    pub digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(default)]
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRecord {
    pub id: TemplateId,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub auto_update: AutoUpdate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<TemplateVersionRecord>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    #[serde(default)]
    pub is_symlink: bool,
    pub size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub mode: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileList {
    pub path: String,
    pub entries: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub path: String,
    pub encoding: String,
    pub content: String,
    pub size: u64,
    #[serde(default)]
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedCommand {
    pub program: String,
    pub argv: Vec<String>,
    pub command: String,
    pub shell: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesiredPower {
    Running,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesiredPort {
    pub allocation_id: AllocationId,
    pub ip: String,
    pub port: u16,
    pub protocol: PortProtocol,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub name: String,
    pub mount_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesiredPod {
    pub id: PodId,
    pub name: String,
    pub image: String,
    pub startup: RenderedCommand,
    pub environment: BTreeMap<String, String>,
    pub limits: Limits,
    pub ports: Vec<DesiredPort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install: Option<InstallSpec>,
    #[serde(default)]
    pub config_files: Vec<ConfigFileSpec>,
    #[serde(default)]
    pub volumes: Vec<VolumeMount>,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    pub stop: StopConfig,
    pub desired_state: DesiredPower,
    pub generation: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub protocol_version: u32,
    pub node_id: NodeId,
    pub revision: i64,
    pub generated_at: DateTime<Utc>,
    pub pods: Vec<DesiredPod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogChunk {
    pub pod_id: PodId,
    pub cursor: String,
    pub lines: Vec<String>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodStats {
    pub pod_id: PodId,
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub memory_limit_bytes: u64,
    pub disk_bytes: u64,
    pub disk_limit_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub pids: u64,
    pub uptime_seconds: u64,
    pub state: PodState,
    pub sampled_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostUpdateStatus {
    #[serde(default)]
    pub supported: bool,
    #[serde(default)]
    pub updates_available: u32,
    #[serde(default)]
    pub reboot_required: bool,
    #[serde(default)]
    pub auto_reboot: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

// runtime groups --------------------------------------------------------

id_type!(RuntimeGroupId);
id_type!(IpAddressId);
id_type!(PortForwardId);
id_type!(ProxyAdapterId);
id_type!(ConfirmationId);
id_type!(WebhookId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeGroup {
    pub id: RuntimeGroupId,
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<TenantId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationStatus {
    Pending,
    Installing,
    Created,
    Started,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionConfirmation {
    pub id: ConfirmationId,
    pub pod_id: PodId,
    pub node_id: NodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_group_id: Option<RuntimeGroupId>,
    pub auto_start: bool,
    pub status: ConfirmationStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
    #[serde(default)]
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// addresses and port forwards ------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpKind {
    Shared,
    Floating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloatingMode {
    Nat,
    Direct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAddress {
    pub id: IpAddressId,
    pub ip: String,
    pub prefix: u8,
    pub kind: IpKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pod_id: Option<PodId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_ip: Option<String>,
    pub mode: FloatingMode,
    #[serde(default)]
    pub primary: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    pub id: PortForwardId,
    pub node_id: NodeId,
    pub pod_id: PodId,
    pub external_ip: String,
    pub external_port: u16,
    pub protocol: PortProtocol,
    pub internal_ip: String,
    pub internal_port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct AddressQuery {
    pub node_id: Option<NodeId>,
    pub pod_id: Option<PodId>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateIp {
    pub ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<u8>,
    pub kind: IpKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pod_id: Option<PodId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<FloatingMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl CreateIp {
    #[must_use]
    pub fn new(ip: impl Into<String>, kind: IpKind) -> Self {
        Self {
            ip: ip.into(),
            prefix: None,
            kind,
            node_id: None,
            pod_id: None,
            target_ip: None,
            mode: None,
            primary: None,
            label: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateIp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pod_id: Option<PodId>,
    #[serde(default)]
    pub unbind: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<FloatingMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub clear_label: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BindIp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_id: Option<IpAddressId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<FloatingMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateForward {
    pub external_ip: String,
    pub external_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<PortProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_ip: Option<String>,
    pub internal_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateForward {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<PortProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub clear_label: bool,
}

// platform --------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlatformSpec {
    #[serde(default)]
    pub prometheus: PrometheusSpec,
    #[serde(default)]
    pub keda: KedaSpec,
    #[serde(default)]
    pub knative: KnativeSpec,
    #[serde(default)]
    pub argocd: ArgoCdSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusSpec {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_metrics_path")]
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default = "default_interval")]
    pub interval: String,
    #[serde(default = "default_scheme")]
    pub scheme: String,
}

impl Default for PrometheusSpec {
    fn default() -> Self {
        Self {
            enabled: false,
            path: default_metrics_path(),
            port: None,
            interval: default_interval(),
            scheme: default_scheme(),
        }
    }
}

fn default_metrics_path() -> String {
    "/metrics".to_string()
}
fn default_interval() -> String {
    "30s".to_string()
}
fn default_scheme() -> String {
    "http".to_string()
}
fn default_max_replicas() -> u32 {
    5
}
fn default_polling() -> u32 {
    30
}
fn default_cooldown() -> u32 {
    300
}
fn default_platform_timeout() -> u32 {
    300
}
fn default_visibility() -> String {
    "external".to_string()
}
fn default_project() -> String {
    "default".to_string()
}
fn default_revision() -> String {
    "HEAD".to_string()
}
fn default_argocd_namespace() -> String {
    "argocd".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KedaSpec {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub min_replicas: u32,
    #[serde(default = "default_max_replicas")]
    pub max_replicas: u32,
    #[serde(default = "default_polling")]
    pub polling_interval_seconds: u32,
    #[serde(default = "default_cooldown")]
    pub cooldown_seconds: u32,
    #[serde(default)]
    pub triggers: Vec<KedaTrigger>,
}

impl Default for KedaSpec {
    fn default() -> Self {
        Self {
            enabled: false,
            min_replicas: 0,
            max_replicas: default_max_replicas(),
            polling_interval_seconds: default_polling(),
            cooldown_seconds: default_cooldown(),
            triggers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KedaTrigger {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric_type: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnativeSpec {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub min_scale: u32,
    #[serde(default = "default_max_replicas")]
    pub max_scale: u32,
    #[serde(default)]
    pub concurrency: u32,
    #[serde(default = "default_platform_timeout")]
    pub timeout_seconds: u32,
    #[serde(default = "default_visibility")]
    pub visibility: String,
}

impl Default for KnativeSpec {
    fn default() -> Self {
        Self {
            enabled: false,
            min_scale: 0,
            max_scale: default_max_replicas(),
            concurrency: 0,
            timeout_seconds: default_platform_timeout(),
            visibility: default_visibility(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgoCdSpec {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_project")]
    pub project: String,
    #[serde(default)]
    pub repo_url: String,
    #[serde(default)]
    pub path: String,
    #[serde(default = "default_revision")]
    pub target_revision: String,
    #[serde(default = "default_argocd_namespace")]
    pub namespace: String,
    #[serde(default)]
    pub manage_workload: bool,
    #[serde(default = "default_true")]
    pub automated: bool,
}

impl Default for ArgoCdSpec {
    fn default() -> Self {
        Self {
            enabled: false,
            project: default_project(),
            repo_url: String::new(),
            path: String::new(),
            target_revision: default_revision(),
            namespace: default_argocd_namespace(),
            manage_workload: false,
            automated: true,
        }
    }
}

// autonomous updates ----------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateSourceKind {
    DockerImage,
    GithubTag,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSource {
    pub kind: UpdateSourceKind,
    pub reference: String,
    #[serde(default)]
    pub tag: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dockerfile: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionMode {
    Sliding,
    PinOldest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activation {
    Immediate,
    HoldLatest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub mode: RetentionMode,
    pub keep: u32,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            mode: RetentionMode::Sliding,
            keep: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolloutPolicy {
    pub steps: Vec<u8>,
    pub step_seconds: u32,
}

impl Default for RolloutPolicy {
    fn default() -> Self {
        Self {
            steps: vec![10, 50, 100],
            step_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicy {
    pub enabled: bool,
    pub interval_seconds: u32,
    pub sources: Vec<UpdateSource>,
    pub rollout: RolloutPolicy,
    pub retention: RetentionPolicy,
    pub activation: Activation,
}

impl Default for UpdatePolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_seconds: 300,
            sources: Vec::new(),
            rollout: RolloutPolicy::default(),
            retention: RetentionPolicy::default(),
            activation: Activation::Immediate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub id: uuid::Uuid,
    pub source: String,
    pub reference: String,
    pub version: String,
    pub image: String,
    pub status: String,
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rollout {
    pub id: uuid::Uuid,
    pub revision_id: uuid::Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_id: Option<uuid::Uuid>,
    pub weight: u8,
    pub status: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub step_index: i32,
}

// metering --------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeterUnit {
    Times,
    Tokens,
}

impl MeterUnit {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            MeterUnit::Times => "times",
            MeterUnit::Tokens => "tokens",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterView {
    pub scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<uuid::Uuid>,
    pub unit: MeterUnit,
}

#[derive(Debug, Clone, Serialize)]
pub struct MeterViewRequest {
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<uuid::Uuid>,
    pub unit: MeterUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumptionLine {
    pub meter: String,
    pub quantity: u64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumption {
    pub unit: MeterUnit,
    pub lines: Vec<ConsumptionLine>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageWindow {
    #[serde(default)]
    pub interval_seconds: u64,
    #[serde(default)]
    pub gaps: u32,
    #[serde(default)]
    pub cpu_millis_seconds: u64,
    #[serde(default)]
    pub memory_byte_seconds: u64,
    #[serde(default)]
    pub disk_byte_seconds: u64,
    #[serde(default)]
    pub io_bytes: u64,
    #[serde(default)]
    pub net_internal_bytes: u64,
    #[serde(default)]
    pub net_external_bytes: u64,
    #[serde(default)]
    pub net_ingress_bytes: u64,
    #[serde(default)]
    pub net_egress_bytes: u64,
    #[serde(default)]
    pub net_unsplit_bytes: u64,
    #[serde(default)]
    pub image_storage_byte_seconds: u64,
    #[serde(default)]
    pub storage_byte_seconds: u64,
    #[serde(default)]
    pub storage_ingress_bytes: u64,
    #[serde(default)]
    pub storage_egress_bytes: u64,
    #[serde(default)]
    pub image_pull_bytes: u64,
    #[serde(default)]
    pub image_pull_count: u64,
    #[serde(default)]
    pub replica_seconds: u64,
    #[serde(default)]
    pub cold_starts: u64,
    #[serde(default)]
    pub at_rest_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodUsage {
    pub pod_id: uuid::Uuid,
    pub unit: MeterUnit,
    pub consumption: Consumption,
    pub usage: UsageWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub subject: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<uuid::Uuid>,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub unit: MeterUnit,
    pub consumption: Consumption,
    pub usage: UsageWindow,
    #[serde(default)]
    pub pods: Vec<PodUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSample {
    pub sampled_at: DateTime<Utc>,
    pub cpu_millis: u32,
    pub memory_bytes: u64,
    pub disk_bytes: u64,
    pub replicas: u32,
    pub at_rest: bool,
    #[serde(default)]
    pub block_read_bytes: u64,
    #[serde(default)]
    pub block_write_bytes: u64,
    #[serde(default)]
    pub net_rx_bytes: u64,
    #[serde(default)]
    pub net_tx_bytes: u64,
    #[serde(default)]
    pub net_internal_rx_bytes: u64,
    #[serde(default)]
    pub net_internal_tx_bytes: u64,
    #[serde(default)]
    pub network_split: bool,
    #[serde(default)]
    pub autoscaler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawUsage {
    pub pod_id: uuid::Uuid,
    #[serde(default)]
    pub samples: Vec<UsageSample>,
}

// billing report ---------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingCoverage {
    pub requested_from_ms: i64,
    pub requested_to_ms: i64,
    pub effective_from_ms: i64,
    pub effective_to_ms: i64,
    #[serde(default)]
    pub first_sample_ms: Option<i64>,
    #[serde(default)]
    pub last_sample_ms: Option<i64>,
    pub interval_seconds: u64,
    pub gaps: u32,
    pub complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingVolumeTotals {
    pub count: u64,
    pub attached_count: u64,
    pub unattached_count: u64,
    pub provisioned_byte_seconds: u64,
    pub used_bytes: u64,
    pub used_byte_seconds: u64,
    pub ingress_bytes: u64,
    pub egress_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingPortTotals {
    pub declared_current: u64,
    pub allocated_current: u64,
    pub forwarded_current: u64,
    pub port_seconds: u64,
    pub created: u64,
    pub updated: u64,
    pub deleted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingBuildTotals {
    pub count: u64,
    pub succeeded: u64,
    pub failed: u64,
    pub running: u64,
    pub queued: u64,
    pub cancelled: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingImageTotals {
    pub count: u64,
    pub stored_bytes: u64,
    pub copy_bytes: u64,
    pub pull_bytes: u64,
    pub pull_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingTotals {
    pub pod_usage: UsageWindow,
    pub volumes: BillingVolumeTotals,
    pub ports: BillingPortTotals,
    pub builds: BillingBuildTotals,
    pub images: BillingImageTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingDeclaredPort {
    pub port_id: uuid::Uuid,
    pub container_port: u16,
    #[serde(default)]
    pub host_ip: Option<String>,
    #[serde(default)]
    pub host_port: Option<u16>,
    pub protocol: String,
    pub exposure: String,
    #[serde(default)]
    pub label: Option<String>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAllocatedPort {
    pub allocation_id: uuid::Uuid,
    pub ip: String,
    pub port: u16,
    pub protocol: String,
    pub primary: bool,
    #[serde(default)]
    pub label: Option<String>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingForwardedPort {
    pub forward_id: uuid::Uuid,
    pub external_ip: String,
    pub external_port: u16,
    pub protocol: String,
    pub internal_ip: String,
    pub internal_port: u16,
    #[serde(default)]
    pub label: Option<String>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingPodPorts {
    pub declared_current: u64,
    pub allocated_current: u64,
    pub forwarded_current: u64,
    pub port_seconds: u64,
    pub created: u64,
    pub updated: u64,
    pub deleted: u64,
    #[serde(default)]
    pub declared: Vec<BillingDeclaredPort>,
    #[serde(default)]
    pub allocated: Vec<BillingAllocatedPort>,
    #[serde(default)]
    pub forwarded: Vec<BillingForwardedPort>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingPod {
    pub pod_id: uuid::Uuid,
    pub resource_group_id: uuid::Uuid,
    pub name: String,
    #[serde(default)]
    pub external_ref: Option<String>,
    #[serde(default)]
    pub reference_id: Option<String>,
    pub coverage: BillingCoverage,
    pub usage: UsageWindow,
    pub ports: BillingPodPorts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingVolumeMount {
    pub pod_id: uuid::Uuid,
    pub mount_path: String,
    pub read_only: bool,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingVolume {
    pub volume_id: uuid::Uuid,
    pub resource_group_id: uuid::Uuid,
    pub name: String,
    pub storage_class: String,
    #[serde(default)]
    pub reference_id: Option<String>,
    pub provisioned_mb: u64,
    pub max_size_mb: u64,
    pub auto_scale: bool,
    pub used_bytes: u64,
    #[serde(default)]
    pub last_used_at_ms: Option<i64>,
    pub attached: bool,
    #[serde(default)]
    pub mounts: Vec<BillingVolumeMount>,
    pub provisioned_byte_seconds: u64,
    pub used_byte_seconds: u64,
    pub ingress_bytes: u64,
    pub egress_bytes: u64,
    pub coverage: BillingCoverage,
    pub used_coverage: BillingCoverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingBuild {
    pub build_id: uuid::Uuid,
    pub source_id: uuid::Uuid,
    pub pod_id: uuid::Uuid,
    pub resource_group_id: uuid::Uuid,
    pub status: String,
    pub reference: String,
    #[serde(default)]
    pub commit_sha: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    pub created_at_ms: i64,
    #[serde(default)]
    pub started_at_ms: Option<i64>,
    #[serde(default)]
    pub finished_at_ms: Option<i64>,
    #[serde(default)]
    pub queue_ms: Option<u64>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    pub coverage: BillingCoverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingImage {
    pub image_id: uuid::Uuid,
    pub reference: String,
    pub digest: String,
    #[serde(default)]
    pub reference_id: Option<String>,
    pub owned_by_resource_group: bool,
    pub stored_bytes: u64,
    pub copy_bytes: u64,
    pub pull_bytes: u64,
    pub pull_count: u64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingResourceGroup {
    pub resource_group_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub reference_id: Option<String>,
    pub coverage: BillingCoverage,
    pub totals: BillingTotals,
    #[serde(default)]
    pub pods: Vec<BillingPod>,
    #[serde(default)]
    pub volumes: Vec<BillingVolume>,
    #[serde(default)]
    pub builds: Vec<BillingBuild>,
    #[serde(default)]
    pub images: Vec<BillingImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingReport {
    pub scope: String,
    pub scope_id: uuid::Uuid,
    pub from_ms: i64,
    pub to_ms: i64,
    pub coverage: BillingCoverage,
    pub totals: BillingTotals,
    #[serde(default)]
    pub resource_groups: Vec<BillingResourceGroup>,
}

// jobs, transfer, webhooks ----------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct JobQuery {
    pub status: Option<JobStatus>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferRequest {
    pub node_id: NodeId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drain_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferAccepted {
    pub job: Job,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookDeliveryStatus {
    Pending,
    Delivered,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSubscription {
    pub id: WebhookId,
    pub url: String,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedWebhook {
    #[serde(flatten)]
    pub webhook: WebhookSubscription,
    pub secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: uuid::Uuid,
    pub subscription_id: WebhookId,
    pub event_type: String,
    #[serde(default)]
    pub payload: serde_json::Value,
    pub status: WebhookDeliveryStatus,
    #[serde(default)]
    pub attempts: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateWebhook {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateWebhook {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

// proxy adapters --------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterProtocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAdapter {
    pub id: ProxyAdapterId,
    pub name: String,
    pub slug: String,
    pub protocol: AdapterProtocol,
    pub endpoint: String,
    pub timeout_ms: u32,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinAdapter {
    pub slug: String,
    pub name: String,
    pub protocol: AdapterProtocol,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdapterList {
    #[serde(default)]
    pub builtin: Vec<BuiltinAdapter>,
    #[serde(default)]
    pub custom: Vec<ProxyAdapter>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateAdapter {
    pub name: String,
    pub slug: String,
    pub protocol: AdapterProtocol,
    pub endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// tenants, resource groups and transfers -------------------------------

id_type!(TenantId);
id_type!(ResourceGroupId);
id_type!(TransferKeyId);
id_type!(TenantTransferId);
id_type!(MeteringProfileId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantStatus {
    Active,
    Suspended,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub description: String,
    pub status: TenantStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(default)]
    pub limits: ApiKeyLimits,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGroup {
    pub id: ResourceGroupId,
    pub tenant_id: TenantId,
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferKeyStatus {
    Active,
    Used,
    Revoked,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferKey {
    pub id: TransferKeyId,
    pub tenant_id: TenantId,
    #[serde(default)]
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_key_id: Option<ApiKeyId>,
    pub status: TransferKeyStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used_by_resource_group_id: Option<ResourceGroupId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedTransferKey {
    #[serde(flatten)]
    pub key: TransferKey,
    pub token: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantTransferStatus {
    Queued,
    Waiting,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantTransfer {
    pub id: TenantTransferId,
    pub resource_group_id: ResourceGroupId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_tenant_id: Option<TenantId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_tenant_id: Option<TenantId>,
    pub status: TenantTransferStatus,
    #[serde(default)]
    pub pending: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dismissed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeterScopeKind {
    Global,
    Container,
    ResourceGroup,
    Tenant,
    DeploymentClass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteringProfile {
    pub id: MeteringProfileId,
    pub scope: MeterScopeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<uuid::Uuid>,
    pub formula: String,
    #[serde(default)]
    pub bill_internal_traffic: bool,
    pub version: i64,
    pub effective_from: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageValue {
    pub scope: String,
    pub scope_id: uuid::Uuid,
    pub from_ms: i64,
    pub to_ms: i64,
    pub unit: String,
    pub times: UsageWindow,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_exact: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cutover_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceAggregate {
    pub reference_id: String,
    pub tenants: Vec<Tenant>,
    pub resource_groups: Vec<ResourceGroup>,
    pub pods: Vec<serde_json::Value>,
}

// pod port bindings --------------------------------------------------

id_type!(PodPortId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortExposure {
    Public,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodPort {
    pub id: PodPortId,
    pub pod_id: PodId,
    pub container_port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_ip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_port: Option<u16>,
    pub protocol: PortProtocol,
    pub exposure: PortExposure,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeConverted {
    pub stack_toml: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

// images ----------------------------------------------------------------

id_type!(ImageId);
id_type!(ImageCopyId);
id_type!(ImagePullId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageSource {
    Pull,
    Upload,
    Build,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: ImageId,
    pub reference: String,
    pub digest: String,
    pub source: ImageSource,
    #[serde(default)]
    pub compressed_bytes: u64,
    #[serde(default)]
    pub stored_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageCopy {
    pub id: ImageCopyId,
    pub image_id: ImageId,
    pub node_id: NodeId,
    #[serde(default)]
    pub size_bytes: u64,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePull {
    pub id: ImagePullId,
    pub image_id: ImageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
    pub registry: String,
    pub reference: String,
    pub digest: String,
    #[serde(default)]
    pub bytes: u64,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

// git -------------------------------------------------------------------

id_type!(GitCredentialId);
id_type!(GitSourceId);
id_type!(BuildId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitProvider {
    Github,
    Gitlab,
    Gitea,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitAuthKind {
    Ssh,
    Token,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCredential {
    pub id: GitCredentialId,
    pub tenant_id: TenantId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    pub name: String,
    pub provider: GitProvider,
    pub auth: GitAuthKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitRefKind {
    Branch,
    Tag,
    Commit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSource {
    pub id: GitSourceId,
    pub resource_group_id: ResourceGroupId,
    pub pod_id: PodId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_id: Option<GitCredentialId>,
    pub name: String,
    pub provider: GitProvider,
    pub repo: String,
    pub reference: String,
    pub ref_kind: GitRefKind,
    pub dockerfile: String,
    pub context: String,
    #[serde(default)]
    pub build_args: BTreeMap<String, String>,
    pub auto_update: bool,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedGitSource {
    #[serde(flatten)]
    pub source: GitSource,
    pub webhook_secret: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBuild {
    pub id: BuildId,
    pub source_id: GitSourceId,
    pub resource_group_id: ResourceGroupId,
    pub pod_id: PodId,
    pub status: BuildStatus,
    pub reference: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default)]
    pub logs: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_id: Option<JobId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision_id: Option<uuid::Uuid>,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<DateTime<Utc>>,
}

// metrics storage -------------------------------------------------------

id_type!(MetricsStoragePolicyId);
id_type!(MetricsArchiveId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStoragePolicy {
    pub id: MetricsStoragePolicyId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<TenantId>,
    pub local_enabled: bool,
    pub local_retention_days: i32,
    pub archive_after_days: i32,
    pub total_retention_days: i32,
    pub local_buffer_minutes: i32,
    pub deletion_grace_days: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_bucket: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_endpoint: Option<String>,
    #[serde(default)]
    pub s3_configured: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsArchive {
    pub id: MetricsArchiveId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<TenantId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pod_id: Option<PodId>,
    pub from_ms: i64,
    pub to_ms: i64,
    pub format: String,
    pub object_key: String,
    pub rows: u64,
    pub bytes: u64,
    pub checksum: String,
    pub schema_version: String,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsArchiveReport {
    #[serde(default)]
    pub tenant_id: Option<uuid::Uuid>,
    pub archived_samples: u64,
    pub archives: u64,
    pub object_keys: Vec<String>,
    pub deleted_local_samples: u64,
    pub deleted_total_samples: u64,
    pub deleted_archives: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeDeployedService {
    pub service: String,
    pub pod_id: uuid::Uuid,
    pub node_id: uuid::Uuid,
    pub image: String,
    #[serde(default)]
    pub ports: Vec<PodPort>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeDeployResult {
    pub stack: String,
    pub pods: Vec<ComposeDeployedService>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

// storage ---------------------------------------------------------------

id_type!(StorageBackendId);
id_type!(VolumeId);
id_type!(PodVolumeMountId);
id_type!(ExternalDatabaseId);
id_type!(PodExternalDatabaseId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackendKind {
    Nfs,
    Smb,
    S3,
    Iscsi,
    Csi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageBackend {
    pub id: StorageBackendId,
    pub tenant_id: TenantId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    pub name: String,
    pub kind: StorageBackendKind,
    pub endpoint: String,
    #[serde(default)]
    pub options: BTreeMap<String, String>,
    #[serde(default)]
    pub secret_configured: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VolumeAccessMode {
    ReadWriteOnce,
    ReadOnlyMany,
    ReadWriteMany,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VolumeStorageClass {
    S3,
    File,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub id: VolumeId,
    pub resource_group_id: ResourceGroupId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend_id: Option<StorageBackendId>,
    #[serde(default = "default_storage_class_client")]
    pub storage_class: VolumeStorageClass,
    pub name: String,
    pub size_mb: u64,
    pub access_mode: VolumeAccessMode,
    #[serde(default = "default_true_client")]
    pub auto_scale: bool,
    #[serde(default = "default_max_volume_client")]
    pub max_size_mb: u64,
    #[serde(default)]
    pub used_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway_token_id: Option<StorageGatewayTokenId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kubernetes_storage_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_storage_class_client() -> VolumeStorageClass {
    VolumeStorageClass::File
}
fn default_true_client() -> bool {
    true
}
fn default_max_volume_client() -> u64 {
    1_048_576
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodVolumeMount {
    pub id: PodVolumeMountId,
    pub pod_id: PodId,
    pub volume_id: VolumeId,
    pub mount_path: String,
    pub read_only: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodVolumeMountView {
    #[serde(flatten)]
    pub mount: PodVolumeMount,
    pub volume: Volume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalDatabaseEngine {
    Mysql,
    Mariadb,
    Postgresql,
    Mongodb,
    Redis,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDatabase {
    pub id: ExternalDatabaseId,
    pub tenant_id: TenantId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    pub name: String,
    pub engine: ExternalDatabaseEngine,
    pub host: String,
    pub port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default)]
    pub password_configured: bool,
    #[serde(default)]
    pub options: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodExternalDatabase {
    pub id: PodExternalDatabaseId,
    pub pod_id: PodId,
    pub database_id: ExternalDatabaseId,
    pub env_prefix: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodExternalDatabaseView {
    #[serde(flatten)]
    pub binding: PodExternalDatabase,
    pub database: ExternalDatabase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTest {
    pub reachable: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodPortEvent {
    pub id: uuid::Uuid,
    pub pod_id: PodId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port_id: Option<PodPortId>,
    pub container_port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_ip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_port: Option<u16>,
    pub protocol: PortProtocol,
    pub exposure: PortExposure,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub action: String,
    pub recorded_at: DateTime<Utc>,
}

id_type!(StorageGatewayTokenId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageGatewayToken {
    pub id: StorageGatewayTokenId,
    pub tenant_id: TenantId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_group_id: Option<ResourceGroupId>,
    pub backend_id: StorageBackendId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_id: Option<VolumeId>,
    #[serde(default)]
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedStorageGatewayToken {
    #[serde(flatten)]
    pub record: StorageGatewayToken,
    pub token: String,
    pub secret_key: String,
}
