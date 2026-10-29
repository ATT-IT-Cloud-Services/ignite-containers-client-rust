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
id_type!(ServerId);
id_type!(JobId);
id_type!(DatabaseHostId);
id_type!(ServerDatabaseId);
id_type!(BackupId);
id_type!(ScheduleId);
id_type!(EmailId);
id_type!(SftpCredentialId);

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
    #[serde(rename = "server:read")]
    ServerRead,
    #[serde(rename = "server:write")]
    ServerWrite,
    #[serde(rename = "server:power")]
    ServerPower,
    #[serde(rename = "server:console")]
    ServerConsole,
    #[serde(rename = "server:files")]
    ServerFiles,
    #[serde(rename = "server:backup")]
    ServerBackup,
    #[serde(rename = "database:read")]
    DatabaseRead,
    #[serde(rename = "database:write")]
    DatabaseWrite,
    #[serde(rename = "server:database")]
    ServerDatabase,
    #[serde(rename = "server:sftp")]
    ServerSftp,
    #[serde(rename = "server:secrets")]
    ServerSecrets,
    #[serde(rename = "notification:send")]
    NotificationSend,
    #[serde(rename = "notification:read")]
    NotificationRead,
    #[serde(rename = "proxy:read")]
    ProxyRead,
    #[serde(rename = "proxy:write")]
    ProxyWrite,
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
    pub server_id: Option<ServerId>,
    #[serde(default)]
    pub primary: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerState {
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
pub struct Server {
    pub id: ServerId,
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
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    pub image: String,
    pub startup: String,
    #[serde(default, skip_serializing)]
    pub environment: BTreeMap<String, String>,
    pub limits: Limits,
    pub state: ServerState,
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
pub struct ServerDetail {
    #[serde(flatten)]
    pub server: Server,
    pub allocations: Vec<Allocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: ApiKeyId,
    pub name: String,
    pub prefix: String,
    pub scopes: ScopeSet,
    #[serde(default)]
    pub server_ids: Vec<ServerId>,
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
pub struct ServerDatabase {
    pub id: ServerDatabaseId,
    pub server_id: ServerId,
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
    pub server_id: ServerId,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub node_id: NodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_id: Option<ServerId>,
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
    pub server_id: ServerId,
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
    pub server_id: ServerId,
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
pub struct ProxyRoute {
    pub id: uuid::Uuid,
    pub domain: String,
    pub upstream: String,
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
pub struct DesiredServer {
    pub id: ServerId,
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
    pub servers: Vec<DesiredServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogChunk {
    pub server_id: ServerId,
    pub cursor: String,
    pub lines: Vec<String>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    pub server_id: ServerId,
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub memory_limit_bytes: u64,
    pub disk_bytes: u64,
    pub disk_limit_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub pids: u64,
    pub uptime_seconds: u64,
    pub state: ServerState,
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
