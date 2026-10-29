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
use ignite_containers_client::{Client, CreateServer, PowerSignal, ServerQuery};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("https://games.example.com", std::env::var("IGNITE_CONTAINERS_API_KEY")?)?;

    let nodes = client.list_nodes().await?;
    let node = nodes.first().expect("no nodes");

    let server = CreateServer::new(node.id, template_id, "survival");
    let created = client.create_server(&server).await?;

    client.power(created.server.id, PowerSignal::Start).await?;

    for server in client.list_servers(ServerQuery { limit: Some(50), ..Default::default() }).await? {
        println!("{} {:?}", server.name, server.state);
    }
    Ok(())
}
```

## Error handling

```rust
match client.get_server(id).await {
    Ok(server) => { /* ... */ }
    Err(ignite_containers_client::Error::Api { status: 404, .. }) => { /* gone */ }
    Err(error) => eprintln!("{error}"),
}
```

## Escape hatch

For endpoints added before the client is regenerated:

```rust
let value: serde_json::Value = client
    .request("GET", "/api/v1/servers?limit=1", None)
    .await?;
```

The client never stores or logs the API key; `Debug` redacts it.

## Management and streaming helpers

The client also exposes first-class helpers for the newer operator surface:

```rust
let env = client.get_server_environment(server_id).await?;
let audit = client.list_audit_events(100).await?;
let state = client.get_node_state(node_id).await?;
let routes = client.list_proxy_routes().await?;
let status = client.proxy_status().await?;
client.reconcile_proxy().await?;
let versions = client.list_template_versions(template_id).await?;
```

`logs_stream(server_id)` returns the authenticated `reqwest::Response` for the SSE
endpoint; consume its byte stream and close it when done. `metrics()` returns the
Prometheus text, while `health()`, `ready()`, `openapi()` and `docs()` cover the
root operational endpoints. Database-host lookup and Rust file `chmod` are also
included.
