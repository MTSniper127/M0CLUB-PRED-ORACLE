
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionLogsConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_client::rpc_response::RpcLogsResponse;
use tokio::sync::mpsc;
use tracing::info;

#[derive(Clone)]
pub struct Subscriber {
    pub rpc: RpcClient,
}

impl Subscriber {
    pub fn new(url: &str) -> Self {
        Self { rpc: RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed()) }
    }

    pub async fn poll_logs(&self, tx: mpsc::Sender<RpcLogsResponse>) -> anyhow::Result<()> {
        // Skeleton poller: fetch recent logs once per interval (not streaming).
        let cfg = RpcTransactionLogsConfig { commitment: Some(CommitmentConfig::confirmed()), ..Default::default() };
        loop {
            let logs = self.rpc.get_recent_performance_samples(1).unwrap_or_default();
            // Fake log entry from samples to drive parser for demo.
            for (i, _s) in logs.iter().enumerate() {
                let r = RpcLogsResponse {
                    signature: format!("SIM_{i}"),
                    err: None,
                    logs: vec!["M0_ORACLE bundle published".into(), "M0_REGISTRY market upsert".into()],
                };
                tx.send(r).await.ok();
            }
            info!("polled logs (simulated)");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }
}
