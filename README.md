# Rust client

`ignite-containers-client` is a standalone async client for the Ignite Containers
control plane. It does **not** depend on any other Ignite Containers crate: all
public request and response types are defined inside the client crate.

```toml
[dependencies]
ignite-containers-client = { git = "https://github.com/ATT-IT-Cloud-Services/ignite-containers-client-rust", tag = "v1.0.0" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
use ignite_containers_client::{Client, CreatePod, PowerSignal, PodQuery};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("https://games.example.com", std::env::var("IGNITE_CONTAINERS_API_KEY")?)?;

    let nodes = client.list_nodes().await?;
    let node = nodes.first().expect("no nodes");

    let pod = CreatePod::new(node.id, template_id, "survival");
    let created = client.create_pod(&pod).await?;

    client.power(created.pod.id, PowerSignal::Start).await?;

    for pod in client.list_pods(PodQuery { limit: Some(50), ..Default::default() }).await? {
        println!("{} {:?}", pod.name, pod.state);
    }
    Ok(())
}
```

## Error handling

```rust
match client.get_pod(id).await {
    Ok(pod) => { /* ... */ }
    Err(ignite_containers_client::Error::Api { status: 404, .. }) => { /* gone */ }
    Err(error) => eprintln!("{error}"),
}
```

## Escape hatch

For endpoints added before the client is regenerated:

```rust
let value: serde_json::Value = client
    .request("GET", "/api/v1/pods?limit=1", None)
    .await?;
```

The client never stores or logs the API key; `Debug` redacts it.

## Management and streaming helpers

The client also exposes first-class helpers for the newer operator surface:

```rust
let env = client.get_pod_environment(pod_id).await?;
let audit = client.list_audit_events(100).await?;
let state = client.get_node_state(node_id).await?;
let routes = client.list_proxy_routes().await?;
let status = client.proxy_status().await?;
client.reconcile_proxy().await?;
let versions = client.list_template_versions(template_id).await?;
```

`logs_stream(pod_id)` returns the authenticated `reqwest::Response` for the SSE
endpoint; consume its byte stream and close it when done. `metrics()` returns the
Prometheus text, while `health()`, `ready()`, `openapi()` and `docs()` cover the
root operational endpoints. Database-host lookup and Rust file `chmod` are also
included.

The remaining operator surface is exposed as typed methods as well:

```rust
let groups = client.list_runtime_groups().await?;
let ips = client.list_ips(AddressQuery::default()).await?;
let platform = client.get_platform(pod_id).await?;
let policy = client.get_update_policy(pod_id).await?;
let revisions = client.list_revisions(pod_id).await?;
let usage = client.pod_usage(pod_id, from, to).await?;
let webhooks = client.list_webhooks().await?;
let jobs = client.list_all_jobs(JobQuery::default()).await?;
let transfer = client.transfer_pod(pod_id, &TransferRequest {
    node_id: target_node,
    mode: Some("copy".into()),
    drain_seconds: None,
}).await?;
let adapters = client.list_proxy_adapters().await?;
let file = client.download_file(pod_id, "server.properties").await?;
client.upload_file(pod_id, "server.properties", file).await?;
```

Floating IPs, port forwards, webhooks, proxy adapters, VNets, interconnects,
listeners and template nests all have first-class methods. See
[`docs/API-COVERAGE.md`](../../docs/API-COVERAGE.md) for the full matrix.

## Billing report

```rust,no_run
let report = client
    .billing_report("tenant", tenant_id, from_ms, to_ms)
    .await?;
for group in &report.resource_groups {
    println!("{:?} {:?}", group.totals.pod_usage, group.totals.volumes);
}
```
