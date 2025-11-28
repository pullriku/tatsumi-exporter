use std::sync::Arc;

use axum::{Router, routing::get};
use prometheus::{IntCounter, IntGaugeVec, TextEncoder, opts};
use tokio::net::ToSocketAddrs;

use crate::metrics;

#[derive(Debug)]
pub struct AppState {
    registry: prometheus::Registry,
    request_counter: prometheus::IntCounter,
    ssh_io_bytes: prometheus::IntGaugeVec,
    container_memory: prometheus::IntGaugeVec,
    container_network: prometheus::IntGaugeVec,
}

pub async fn serve(
    bind_addr: impl ToSocketAddrs,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = make_state();

    let app = Router::new()
        .route("/metrics", get(metrics))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn make_state() -> AppState {
    let registry = prometheus::Registry::new();

    let request_counter =
        IntCounter::new("request_count", "Number of requests to the server.")
            .unwrap();
    registry
        .register(Box::new(request_counter.clone()))
        .unwrap();

    let ssh_io_bytes = IntGaugeVec::new(
        opts!("ssh_io_bytes", "SSH I/O bytes"),
        &["direction"],
    )
    .unwrap();
    registry.register(Box::new(ssh_io_bytes.clone())).unwrap();

    let container_memory = IntGaugeVec::new(
        opts!("container_memory", "Container memory usage"),
        &["container"],
    )
    .unwrap();
    registry
        .register(Box::new(container_memory.clone()))
        .unwrap();

    let container_network = IntGaugeVec::new(
        opts!("container_network", "Container network usage"),
        &["container", "direction"],
    )
    .unwrap();
    registry
        .register(Box::new(container_network.clone()))
        .unwrap();

    AppState {
        registry,
        request_counter,
        ssh_io_bytes,
        container_memory,
        container_network,
    }
}

async fn metrics(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> String {
    // let state =  state.as_ref();

    state.request_counter.inc();

    if let Ok(ssh) = metrics::ssh::collect_ssh_metrics() {
        state
            .ssh_io_bytes
            .with_label_values(&["read"])
            .set(ssh.read_bytes as i64);
        state
            .ssh_io_bytes
            .with_label_values(&["write"])
            .set(ssh.write_bytes as i64);
    }

    if let Ok(containers) =
        metrics::container::collect_container_metrics().await
    {
        for metric in containers {
            state
                .container_memory
                .with_label_values(&[&metric.name])
                .set(metric.mem_bytes as i64);

            state
                .container_network
                .with_label_values(&[metric.name.as_str(), "rx"])
                .set(metric.rx_bytes as i64);
            state
                .container_network
                .with_label_values(&[metric.name.as_str(), "tx"])
                .set(metric.tx_bytes as i64);
        }
    }

    let encoder = TextEncoder::new();
    let metric_families = state.registry.gather();
    let mut buffer = String::new();
    encoder.encode_utf8(&metric_families, &mut buffer).unwrap();

    buffer
}
