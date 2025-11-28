use procfs::process::all_processes;

pub struct SshMetrics {
    pub read_bytes: u64,
    pub write_bytes: u64,
}

pub fn collect_ssh_metrics() -> anyhow::Result<SshMetrics> {
    let (read_bytes, write_bytes) = all_processes()?
        .flatten()
        .filter(|p| matches!(p.stat(), Ok(stat) if stat.comm.contains("sshd")))
        .flat_map(|p| p.io())
        .fold((0, 0), |(read_bytes, write_bytes), io| {
            (read_bytes + io.read_bytes, write_bytes + io.write_bytes)
        });

    Ok(SshMetrics {
        read_bytes,
        write_bytes,
    })
}
