use bollard::{
    Docker,
    query_parameters::{InspectContainerOptions, ListContainersOptions},
};
use procfs::process::Process;

#[derive(Debug)]
pub struct ContainerMetric {
    pub name: String,
    pub pid: i32,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub cpu_ticks: u64, // CPU (procfs) -> utime + stime
    pub mem_bytes: u64, // Memory (procfs) -> rss * page_size
}

pub async fn collect_container_metrics() -> anyhow::Result<Vec<ContainerMetric>>
{
    let docker = Docker::connect_with_local_defaults()?;

    let option = ListContainersOptions::default();
    let containers = docker.list_containers(Some(option));

    let mut metrics = Vec::new();

    for container in containers.await? {
        // コンテナIDから詳細情報を取得（PIDを知るため）
        let id = container.id.unwrap_or_default();
        let inspect = docker
            .inspect_container(&id, None::<InspectContainerOptions>)
            .await?;

        let name = inspect.name.unwrap_or_else(|| "unknown".to_string());

        // PIDを取得
        if let Some(state) = inspect.state
            && let Some(pid) = state.pid
            && pid > 0
            && let Ok(process) = Process::new(pid as i32)
        {
            let (rx, tx) = match process.dev_status() {
                Ok(dev_stats) => {
                    dev_stats.into_iter().fold((0, 0), |(r, t), (_, stat)| {
                        (r + stat.recv_bytes, t + stat.sent_bytes)
                    })
                }
                Err(_) => (0, 0),
            };

            let (cpu_ticks, mem_bytes) = if let Ok(stat) = process.stat() {
                let page_size = procfs::page_size();
                (stat.utime + stat.stime, stat.rss * page_size)
            } else {
                (0, 0)
            };

            metrics.push(ContainerMetric {
                name,
                pid: pid as i32,
                rx_bytes: rx,
                tx_bytes: tx,
                cpu_ticks,
                mem_bytes,
            });
        }
    }

    Ok(metrics)
}
