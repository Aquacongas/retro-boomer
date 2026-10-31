//! Mining controls own only the external PoW worker, never the node or wallet key.
use super::*;
use crate::model::{MinedBlocksSnapshot, MiningSnapshot};

#[derive(Debug, Default)]
pub(super) struct MinerProcess {
    child: Option<Child>,
    log_path: Option<PathBuf>,
    payout_address: Option<String>,
}

impl MinerProcess {
    fn stop(&mut self) -> Result<(), String> {
        if let Some(child) = self.child.as_mut() {
            if child.try_wait().map_err(|e| e.to_string())?.is_none() {
                child.kill().map_err(|e| format!("stop GUI miner: {e}"))?;
            }
            child
                .wait()
                .map_err(|e| format!("wait for GUI miner: {e}"))?;
            self.child = None;
        }
        Ok(())
    }
}
impl Drop for MinerProcess {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[derive(Deserialize)]
struct MiningNodeStatus {
    mining: bool,
    mining_ready: bool,
    mining_confirmed_peers: usize,
    mining_required_peers: usize,
    isolated_mining: bool,
}

impl Backend {
    pub fn mining_threads_available(&self) -> usize {
        available_threads()
    }

    pub fn stop_gui_miner(&self) -> Result<(), String> {
        self.inner
            .miner
            .lock()
            .map_err(|_| "Miner lock is poisoned".to_string())?
            .stop()
    }

    pub async fn start_gui_miner(&self, threads: usize) -> Result<(), String> {
        if self.is_mock() {
            return Err("Mining requires a real local node.".into());
        }
        let config = self.config_snapshot()?;
        // Attach to the existing daemon; this method must never call ensure_running/spawn.
        let status: MiningNodeStatus = self.rpc("getNodeStatus", json!([])).await?;
        if status.mining {
            return Err("The node already owns a built-in miner. GUI START/STOP cannot control it; no second miner was started.".into());
        }
        if !config.data_dir.join("wallet.key").is_file() {
            return Err("Existing wallet.key is required in the selected data directory; mining never creates or replaces it.".into());
        }
        let payout: WalletAddressInfo = self.rpc("walletActiveAddress", json!([])).await?;
        let key_path = mining_key_path(&config.data_dir);
        ensure_mining_key(&key_path)?;
        let binary = config.node_binary.with_file_name(if cfg!(windows) {
            "retro-boomer-miner.exe"
        } else {
            "retro-boomer-miner"
        });
        if !binary.is_file() {
            return Err(format!("External miner is missing: {}", binary.display()));
        }
        let mut state = self
            .inner
            .miner
            .lock()
            .map_err(|_| "Miner lock is poisoned".to_string())?;
        if let Some(child) = state.child.as_mut() {
            if child.try_wait().map_err(|e| e.to_string())?.is_none() {
                return Err("GUI miner is already running.".into());
            }
            child.wait().map_err(|e| e.to_string())?;
            state.child = None;
        }
        let log_path = config.data_dir.join("retro-boomer-miner.log");
        let log = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_path)
            .map_err(|e| format!("open miner log: {e}"))?;
        let error_log = log.try_clone().map_err(|e| e.to_string())?;
        let mut command = Command::new(binary);
        command
            .arg("--rpc")
            .arg(&config.rpc_url)
            .arg("--key-file")
            .arg(key_path)
            .arg("--coinbase")
            .arg(&payout.address)
            .arg("--threads")
            .arg(threads.clamp(1, available_threads()).to_string())
            .arg("--rpc-timeout")
            .arg("3600")
            // Bind this worker to the real active wallet address captured at START.
            .stdin(Stdio::null())
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(error_log))
            .env("NO_COLOR", "1");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        state.child = Some(
            command
                .spawn()
                .map_err(|e| format!("start external miner: {e}"))?,
        );
        state.log_path = Some(log_path);
        state.payout_address = Some(payout.address);
        Ok(())
    }

    pub async fn mining_snapshot(
        &self,
        page: u32,
        threads: usize,
    ) -> Result<MiningSnapshot, String> {
        if self.is_mock() {
            return Err("Mining requires a real local node.".into());
        }
        let (status, history): (MiningNodeStatus, MinedBlocksSnapshot) = tokio::try_join!(
            self.rpc("getNodeStatus", json!([])),
            self.rpc("walletMinedBlocks", json!([page.max(1), 10])),
        )?;
        let (mut enabled, log_path, mut error, payout_address) = {
            let mut state = self
                .inner
                .miner
                .lock()
                .map_err(|_| "Miner lock is poisoned".to_string())?;
            let mut error = None;
            let enabled = if let Some(child) = state.child.as_mut() {
                match child.try_wait().map_err(|e| e.to_string())? {
                    None => true,
                    Some(exit) => {
                        error = Some(format!("GUI miner exited: {exit}"));
                        state.child = None;
                        false
                    }
                }
            } else {
                false
            };
            (
                enabled,
                state.log_path.clone(),
                error,
                state.payout_address.clone(),
            )
        };
        if enabled || error.is_some() {
            if let Some(path) = log_path {
                if let Ok(tail) = read_node_log_tail(&path, 8192, 8).await {
                    if let Some(line) = tail.lines().rev().find(|line| {
                        line.contains("external mining API is disabled")
                            || line.contains("401 Unauthorized")
                            || line.contains("403 Forbidden")
                            || line.contains("miner_address must match")
                    }) {
                        // A worker with rejected API access must not spin indefinitely.
                        self.stop_gui_miner()?;
                        enabled = false;
                        error = Some(format!("Mining API rejected the worker: {line}. This running node needs extminer mode and the same mining.key token. Restart it manually when convenient; the GUI has not restarted it."));
                    } else if !enabled && error.is_some() && !tail.is_empty() {
                        error = Some(tail);
                    }
                }
            }
        }
        if status.mining && !enabled {
            error = Some("The existing node is mining internally. GUI START/STOP does not stop or reconfigure an externally managed miner.".into());
        }
        Ok(MiningSnapshot {
            enabled,
            ready: status.mining_ready,
            isolated: status.isolated_mining,
            confirmed_peers: status.mining_confirmed_peers,
            required_peers: status.mining_required_peers,
            selected_threads: threads.clamp(1, available_threads()),
            available_threads: available_threads(),
            history,
            error,
            node_mining: status.mining,
            payout_address: if enabled { payout_address } else { None },
        })
    }
}

/// The path, not the credential, may be passed to child processes.
pub(super) fn mining_key_path(data_dir: &Path) -> PathBuf {
    std::env::var_os("RETRO_BOOMER_MINING_KEY_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir.join("mining.key"))
}

fn validate_mining_key(path: &Path) -> Result<(), String> {
    use std::io::Read;
    let before = std::fs::symlink_metadata(path).map_err(|e| format!("inspect mining key: {e}"))?;
    if !before.is_file() || before.file_type().is_symlink() {
        return Err("Mining key must be a regular file, not a symlink.".into());
    }
    let file = std::fs::File::open(path).map_err(|e| format!("open mining key: {e}"))?;
    let meta = file
        .metadata()
        .map_err(|e| format!("inspect open mining key: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        // uid_t is u32 on the supported Linux/macOS targets. No additional crate.
        unsafe extern "C" {
            fn geteuid() -> u32;
        }
        // SAFETY: geteuid only reads the effective process identity.
        let uid = unsafe { geteuid() };
        if meta.uid() != uid || meta.permissions().mode() & 0o777 != 0o600 {
            return Err("Mining key must belong to this user and have permissions 0600. Existing files are never changed.".into());
        }
        if before.dev() != meta.dev() || before.ino() != meta.ino() {
            return Err("Mining key changed while opening it.".into());
        }
    }
    if !meta.is_file() || meta.len() > 4096 {
        return Err("Mining key must be a regular file no larger than 4096 bytes.".into());
    }
    let mut token = Zeroizing::new(String::new());
    file.take(4097)
        .read_to_string(&mut token)
        .map_err(|_| "Mining key is not valid text.".to_string())?;
    let token = token.trim();
    if token.len() < 16
        || token.len() > 4096
        || token
            .bytes()
            .any(|b| b.is_ascii_whitespace() || b.is_ascii_control())
    {
        return Err("Mining key has an invalid format. Existing files are never replaced.".into());
    }
    Ok(())
}

/// Publish a fully written and synced token atomically, without replacing a winner.
/// hard_link is the atomic no-clobber operation; rename would overwrite on Unix.
pub(super) fn ensure_mining_key(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => return validate_mining_key(path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("inspect mining key: {e}")),
    }
    let parent = path.parent().ok_or("Mining key has no parent directory.")?;
    let mut random = Zeroizing::new([0u8; 32]);
    OsRng
        .try_fill_bytes(&mut *random)
        .map_err(|_| "OS randomness unavailable for mining API token.".to_string())?;
    let token = Zeroizing::new(hex::encode(&*random));
    let mut suffix = [0u8; 16];
    OsRng
        .try_fill_bytes(&mut suffix)
        .map_err(|_| "OS randomness unavailable for atomic key publication.".to_string())?;
    let temporary = parent.join(format!(".mining-key-{}.tmp", hex::encode(suffix)));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .map_err(|e| format!("create private mining key: {e}"))?;
    let result = (|| {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .map_err(|e| format!("set new mining key permissions: {e}"))?;
        }
        file.write_all(token.as_bytes())
            .map_err(|e| format!("write mining key: {e}"))?;
        file.write_all(b"\n")
            .map_err(|e| format!("finish mining key: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("sync mining key: {e}"))?;
        match std::fs::hard_link(&temporary, path) {
            Ok(()) => {
                #[cfg(unix)]
                std::fs::File::open(parent)
                    .and_then(|dir| dir.sync_all())
                    .map_err(|e| format!("sync mining key directory: {e}"))?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(format!("publish mining key without overwriting: {e}")),
        }
        validate_mining_key(path)
    })();
    drop(file);
    let cleanup = std::fs::remove_file(&temporary);
    result?;
    cleanup.map_err(|e| format!("remove private mining key staging file: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_run_publishes_private_256_bit_token_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mining.key");
        ensure_mining_key(&path).unwrap();
        let token = std::fs::read_to_string(&path).unwrap();
        assert_eq!(hex::decode(token.trim()).unwrap().len(), 32);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        assert!(!dir.path().join("wallet.key").exists());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
        let second = dir.path().join("second.key");
        ensure_mining_key(&second).unwrap();
        assert_ne!(
            std::fs::read(&path).unwrap(),
            std::fs::read(second).unwrap()
        );
    }

    #[test]
    fn existing_token_and_wallet_are_never_changed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mining.key");
        ensure_mining_key(&path).unwrap();
        let before = std::fs::read(&path).unwrap();
        let wallet = dir.path().join("wallet.key");
        std::fs::write(&wallet, b"existing wallet preservation sentinel").unwrap();
        ensure_mining_key(&path).unwrap();
        assert_eq!(before, std::fs::read(path).unwrap());
        assert_eq!(
            std::fs::read(wallet).unwrap(),
            b"existing wallet preservation sentinel"
        );
    }

    #[test]
    #[cfg(unix)]
    fn bad_permissions_and_symlinks_are_rejected_without_repair() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mining.key");
        ensure_mining_key(&path).unwrap();
        let before = std::fs::read(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(ensure_mining_key(&path).unwrap_err().contains("0600"));
        assert_eq!(before, std::fs::read(&path).unwrap());
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o644
        );
        let link = dir.path().join("linked.key");
        symlink(&path, &link).unwrap();
        assert!(ensure_mining_key(&link).is_err());
    }

    #[test]
    fn concurrent_first_runs_never_replace_the_winning_token() {
        let dir = tempfile::tempdir().unwrap();
        let path = std::sync::Arc::new(dir.path().join("mining.key"));
        let workers: Vec<_> = (0..8)
            .map(|_| {
                let path = path.clone();
                std::thread::spawn(move || {
                    ensure_mining_key(&path).unwrap();
                    std::fs::read(&*path).unwrap()
                })
            })
            .collect();
        let values: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
        assert!(values.iter().all(|value| value == &values[0]));
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    fn backend_for_test(url: String, dir: &Path) -> Backend {
        Backend {
            inner: Arc::new(BackendInner {
                config: Mutex::new(BackendConfig {
                    rpc_url: url,
                    rpc_listen: "127.0.0.1:0".into(),
                    p2p_listen: "127.0.0.1:0".into(),
                    data_dir: dir.into(),
                    node_binary: dir.join("must-not-spawn"),
                    seeds: vec![],
                    log_level: LogLevel::Info,
                    language: None,
                    media_downloads: false,
                    address_labels: BTreeMap::new(),
                    settings_path: dir.join("settings.json"),
                    mock: false,
                }),
                client: Client::new(),
                next_request_id: AtomicU64::new(1),
                supervisor: Mutex::new(SupervisorState {
                    child: None,
                    owned: false,
                    desired_mode: NodeMode::Extminer,
                    selected_threads: 1,
                    genesis: false,
                }),
                miner: Mutex::new(MinerProcess::default()),
                system: Mutex::new(System::new()),
                wallet_utxo_cache: tokio::sync::Mutex::new(None),
                native_media: tokio::sync::Mutex::new(native_media::Maintenance::default()),
                media_downloads: tokio::sync::watch::channel(false).0,
            }),
        }
    }

    // A bounded RPC fixture exercises the existing-node branch without starting a
    // real node, generating proofs, or touching any real wallet directory.
    async fn rpc_fixture(
        method: &'static str,
        result: Value,
    ) -> (String, tokio::task::JoinHandle<()>) {
        use tokio::io::{AsyncBufReadExt, BufReader};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(socket);
            let mut len = 0;
            loop {
                let mut line = String::new();
                assert!(reader.read_line(&mut line).await.unwrap() > 0);
                if line == "\r\n" {
                    break;
                }
                if let Some((name, value)) = line.split_once(':') {
                    if name.eq_ignore_ascii_case("content-length") {
                        len = value.trim().parse().unwrap();
                    }
                }
            }
            let mut body = vec![0; len];
            reader.read_exact(&mut body).await.unwrap();
            let request: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(request["method"], format!("retro_boomer_{method}"));
            let body = json!({"jsonrpc":"2.0", "id":request["id"], "result":result}).to_string();
            reader.into_inner().write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
        });
        (url, task)
    }

    #[tokio::test]
    async fn running_node_is_attached_without_spawn_or_key_creation() {
        let dir = tempfile::tempdir().unwrap();
        let (url, server) = rpc_fixture("blockCount", json!(7)).await;
        let backend = backend_for_test(url, dir.path());
        backend.ensure_running().await.unwrap();
        server.await.unwrap();
        assert!(!backend.lock_supervisor().unwrap().owned);
        assert!(backend.lock_supervisor().unwrap().child.is_none());
        assert!(backend.inner.miner.lock().unwrap().child.is_none());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[tokio::test]
    async fn existing_builtin_miner_is_never_restarted_or_duplicated() {
        let dir = tempfile::tempdir().unwrap();
        let (url, server) = rpc_fixture(
            "getNodeStatus",
            json!({
                "mining":true, "mining_ready":true, "mining_confirmed_peers":1,
                "mining_required_peers":1, "isolated_mining":false,
            }),
        )
        .await;
        let backend = backend_for_test(url, dir.path());
        assert!(backend
            .start_gui_miner(1)
            .await
            .unwrap_err()
            .contains("already owns"));
        server.await.unwrap();
        assert!(backend.inner.miner.lock().unwrap().child.is_none());
        assert!(backend.lock_supervisor().unwrap().child.is_none());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    #[cfg(unix)]
    fn stopping_worker_leaves_node_process_alive() {
        let dir = tempfile::tempdir().unwrap();
        let backend = backend_for_test("http://127.0.0.1:0".into(), dir.path());
        // Real OS child-process lifecycle test; no node or mining executable.
        let node = Command::new("sleep").arg("60").spawn().unwrap();
        {
            let mut state = backend.lock_supervisor().unwrap();
            state.child = Some(node);
            state.owned = true;
        }
        backend.inner.miner.lock().unwrap().child =
            Some(Command::new("sleep").arg("60").spawn().unwrap());
        backend.stop_gui_miner().unwrap();
        backend.stop_gui_miner().unwrap();
        assert!(backend.inner.miner.lock().unwrap().child.is_none());
        assert!(backend
            .lock_supervisor()
            .unwrap()
            .child
            .as_mut()
            .unwrap()
            .try_wait()
            .unwrap()
            .is_none());
    }
}
