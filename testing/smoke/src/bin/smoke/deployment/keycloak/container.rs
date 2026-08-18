use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use veoveo_deploy_contract::LoadedProfile;

use super::{
    CERT_MOUNT_DESTINATION, KEY_MOUNT_DESTINATION, KEYCLOAK_CONTAINER_NAME, KEYCLOAK_DISCOVERY_URL,
    KEYCLOAK_ENV_VARS, KEYCLOAK_IMAGE, KEYCLOAK_ISSUER, KEYCLOAK_REALM_NAME,
    LOCAL_KEYCLOAK_CONFIG_DIGEST_LABEL, REALM_MOUNT_DESTINATION, startup_args,
    tls::{StateLock, TlsGeneration, ensure_generation},
};
use crate::deployment::{output_checked, path_str, status_checked};

#[derive(Debug)]
pub(super) struct FingerprintInputs<'a> {
    pub(super) image: &'a str,
    pub(super) realm_bytes: &'a [u8],
    pub(super) startup_args: &'a [String],
    pub(super) env_vars: &'a [(&'a str, &'a str)],
    pub(super) server_cert_pem: &'a [u8],
    pub(super) active_generation_digest: &'a str,
    pub(super) mount_destinations: &'a [&'a str],
    pub(super) network: &'a str,
    pub(super) restart_policy: &'a str,
}

pub(super) fn config_fingerprint(inputs: &FingerprintInputs<'_>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"veoveo.io/local-keycloak-config/v1\0");
    hasher.update(inputs.image.as_bytes());
    hasher.update([0]);
    hasher.update(inputs.realm_bytes);
    hasher.update([0]);
    for arg in inputs.startup_args {
        hasher.update(arg.as_bytes());
        hasher.update([0]);
    }
    for (key, value) in inputs.env_vars {
        hasher.update(key.as_bytes());
        hasher.update(b"=");
        hasher.update(value.as_bytes());
        hasher.update([0]);
    }
    hasher.update(inputs.server_cert_pem);
    hasher.update([0]);
    hasher.update(inputs.active_generation_digest.as_bytes());
    hasher.update([0]);
    for destination in inputs.mount_destinations {
        hasher.update(destination.as_bytes());
        hasher.update([0]);
    }
    hasher.update(inputs.network.as_bytes());
    hasher.update([0]);
    hasher.update(inputs.restart_policy.as_bytes());
    hex::encode(hasher.finalize())
}

fn is_container_running(name: &str) -> Result<bool> {
    let filter = format!("name=^/{name}$");
    let output = output_checked(
        "docker",
        ["ps", "--filter", &filter, "--format", "{{.Names}}"],
        None,
    )?;
    Ok(!String::from_utf8_lossy(&output).trim().is_empty())
}

fn container_exists(name: &str) -> Result<bool> {
    let filter = format!("name=^/{name}$");
    let output = output_checked(
        "docker",
        ["ps", "-a", "--filter", &filter, "--format", "{{.Names}}"],
        None,
    )?;
    Ok(!String::from_utf8_lossy(&output).trim().is_empty())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContainerReconcileDecision {
    Create,
    Start,
    Reuse,
    Recreate,
}

pub(super) fn container_reconcile_decision(
    exists: bool,
    valid: bool,
    running: bool,
) -> ContainerReconcileDecision {
    if !exists {
        ContainerReconcileDecision::Create
    } else if !valid {
        ContainerReconcileDecision::Recreate
    } else if !running {
        ContainerReconcileDecision::Start
    } else {
        ContainerReconcileDecision::Reuse
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct ContainerInspect {
    #[serde(rename = "Config")]
    pub(super) config: ContainerInspectConfig,
    #[serde(rename = "Mounts")]
    pub(super) mounts: Vec<ContainerInspectMount>,
    #[serde(rename = "NetworkSettings")]
    pub(super) network_settings: ContainerInspectNetworkSettings,
    #[serde(rename = "HostConfig", default)]
    pub(super) host_config: Option<ContainerInspectHostConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct ContainerInspectHostConfig {
    #[serde(rename = "RestartPolicy", default)]
    pub(super) restart_policy: Option<ContainerInspectRestartPolicy>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct ContainerInspectRestartPolicy {
    #[serde(rename = "Name", default)]
    pub(super) name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct ContainerInspectConfig {
    #[serde(rename = "Image")]
    pub(super) image: String,
    #[serde(rename = "Labels", default)]
    pub(super) labels: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct ContainerInspectMount {
    #[serde(rename = "Source")]
    pub(super) source: String,
    #[serde(rename = "Destination")]
    pub(super) destination: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct ContainerInspectNetworkSettings {
    #[serde(rename = "Networks")]
    pub(super) networks: BTreeMap<String, Value>,
}

fn inspect_container(name: &str) -> Result<ContainerInspect> {
    let output = output_checked("docker", ["inspect", name], None)?;
    let mut parsed: Vec<ContainerInspect> = serde_json::from_slice(&output)
        .with_context(|| format!("decoding docker inspect output for {name}"))?;
    parsed
        .pop()
        .with_context(|| format!("docker inspect returned no container named {name}"))
}

pub(super) fn container_is_valid(
    inspect: &ContainerInspect,
    network: &str,
    expected_mounts: &[(&Path, &str)],
    expected_fingerprint: &str,
) -> Result<bool> {
    if inspect.config.image != KEYCLOAK_IMAGE {
        return Ok(false);
    }
    let stored_fingerprint = inspect
        .config
        .labels
        .as_ref()
        .and_then(|labels| labels.get(LOCAL_KEYCLOAK_CONFIG_DIGEST_LABEL));
    if stored_fingerprint.map(String::as_str) != Some(expected_fingerprint) {
        return Ok(false);
    }
    let restart_policy = inspect
        .host_config
        .as_ref()
        .and_then(|host_config| host_config.restart_policy.as_ref())
        .map(|policy| policy.name.as_str());
    if restart_policy != Some(super::LOCAL_KEYCLOAK_RESTART_POLICY) {
        return Ok(false);
    }
    if !inspect.network_settings.networks.contains_key(network) {
        return Ok(false);
    }
    for (source, destination) in expected_mounts {
        let source = path_str(source)?;
        let matches = inspect
            .mounts
            .iter()
            .any(|mount| mount.source == source && mount.destination == *destination);
        if !matches {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn ensure(profile: &LoadedProfile, lock: &StateLock) -> Result<()> {
    let cluster = profile
        .definition
        .kubernetes
        .local_cluster
        .as_ref()
        .context("deployment profile requiring local keycloak does not manage a local cluster")?;
    let network = format!("k3d-{}", cluster.name);
    let generation = ensure_generation(&profile.repository, lock)?;
    let realm_path = profile
        .repository
        .join("configs/keycloak/veoveo-local-realm.json");
    ensure!(
        realm_path.is_file(),
        "local Keycloak realm fixture {} does not exist",
        realm_path.display()
    );
    let realm_bytes =
        fs::read(&realm_path).with_context(|| format!("reading {}", realm_path.display()))?;
    let server_cert_pem = fs::read(&generation.cert_path)
        .with_context(|| format!("reading {}", generation.cert_path.display()))?;
    let args = startup_args();
    let mount_destinations = [
        REALM_MOUNT_DESTINATION,
        CERT_MOUNT_DESTINATION,
        KEY_MOUNT_DESTINATION,
    ];
    let expected_fingerprint = config_fingerprint(&FingerprintInputs {
        image: KEYCLOAK_IMAGE,
        realm_bytes: &realm_bytes,
        startup_args: &args,
        env_vars: KEYCLOAK_ENV_VARS,
        server_cert_pem: &server_cert_pem,
        active_generation_digest: &generation.digest,
        mount_destinations: &mount_destinations,
        network: &network,
        restart_policy: super::LOCAL_KEYCLOAK_RESTART_POLICY,
    });
    let expected_mounts = [
        (realm_path.as_path(), REALM_MOUNT_DESTINATION),
        (generation.cert_path.as_path(), CERT_MOUNT_DESTINATION),
        (generation.key_path.as_path(), KEY_MOUNT_DESTINATION),
    ];

    if container_exists(KEYCLOAK_CONTAINER_NAME)? {
        let inspect = inspect_container(KEYCLOAK_CONTAINER_NAME)?;
        let valid =
            container_is_valid(&inspect, &network, &expected_mounts, &expected_fingerprint)?;
        let decision = container_reconcile_decision(
            true,
            valid,
            is_container_running(KEYCLOAK_CONTAINER_NAME)?,
        );
        match decision {
            ContainerReconcileDecision::Start | ContainerReconcileDecision::Reuse => {
                if matches!(decision, ContainerReconcileDecision::Start) {
                    status_checked("docker", ["start", KEYCLOAK_CONTAINER_NAME], &[], None)?;
                }
                match wait_until_ready(&cluster.name, &generation, Duration::from_secs(120)) {
                    Ok(()) => {
                        println!(
                            "Local Keycloak {KEYCLOAK_CONTAINER_NAME} is ready at {KEYCLOAK_ISSUER}"
                        );
                        return Ok(());
                    }
                    Err(error) => {
                        println!(
                            "Local Keycloak {KEYCLOAK_CONTAINER_NAME} failed readiness and will be recreated: {error:#}"
                        );
                        status_checked("docker", ["rm", "-f", KEYCLOAK_CONTAINER_NAME], &[], None)?;
                    }
                }
            }
            ContainerReconcileDecision::Recreate => {
                println!(
                    "Local Keycloak {KEYCLOAK_CONTAINER_NAME} configuration is stale (image, realm/TLS configuration, network, mounts, or restart policy changed) and will be recreated"
                );
                status_checked("docker", ["rm", "-f", KEYCLOAK_CONTAINER_NAME], &[], None)?;
            }
            ContainerReconcileDecision::Create => unreachable!("existing container was inspected"),
        }
    }

    create_container(&network, &realm_path, &generation, &expected_fingerprint)?;
    wait_until_ready(&cluster.name, &generation, Duration::from_secs(120))?;
    println!("Local Keycloak {KEYCLOAK_CONTAINER_NAME} is ready at {KEYCLOAK_ISSUER}");
    Ok(())
}

fn create_container(
    network: &str,
    realm_path: &Path,
    generation: &TlsGeneration,
    fingerprint: &str,
) -> Result<()> {
    let args = create_container_arguments(network, realm_path, generation, fingerprint)?;
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    status_checked("docker", refs, &[], None)
}

pub(super) fn create_container_arguments(
    network: &str,
    realm_path: &Path,
    generation: &TlsGeneration,
    fingerprint: &str,
) -> Result<Vec<String>> {
    let realm_mount = format!("{}:{REALM_MOUNT_DESTINATION}:ro", path_str(realm_path)?);
    let crt_mount = format!(
        "{}:{CERT_MOUNT_DESTINATION}:ro",
        path_str(&generation.cert_path)?
    );
    let key_mount = format!(
        "{}:{KEY_MOUNT_DESTINATION}:ro",
        path_str(&generation.key_path)?
    );
    let label = format!("{LOCAL_KEYCLOAK_CONFIG_DIGEST_LABEL}={fingerprint}");
    let mut args = vec![
        "run".to_owned(),
        "-d".to_owned(),
        "--name".to_owned(),
        KEYCLOAK_CONTAINER_NAME.to_owned(),
        "--restart".to_owned(),
        super::LOCAL_KEYCLOAK_RESTART_POLICY.to_owned(),
        "--network".to_owned(),
        network.to_owned(),
        "-p".to_owned(),
        "127.0.0.1:8080:8080".to_owned(),
        "--label".to_owned(),
        label,
    ];
    for (key, value) in KEYCLOAK_ENV_VARS {
        args.push("-e".to_owned());
        args.push(format!("{key}={value}"));
    }
    args.extend([
        "-v".to_owned(),
        realm_mount,
        "-v".to_owned(),
        crt_mount,
        "-v".to_owned(),
        key_mount,
        KEYCLOAK_IMAGE.to_owned(),
    ]);
    args.extend(startup_args());
    Ok(args)
}

fn wait_until_ready(
    cluster_name: &str,
    generation: &TlsGeneration,
    timeout: Duration,
) -> Result<()> {
    let deadline = Instant::now() + timeout;
    let mut last_error: Option<anyhow::Error> = None;
    while Instant::now() < deadline {
        match check_ready(cluster_name, &generation.ca_path) {
            Ok(()) => return Ok(()),
            Err(error) => last_error = Some(error),
        }
        thread::sleep(Duration::from_millis(500));
    }
    let logs = output_checked(
        "docker",
        ["logs", "--tail", "200", KEYCLOAK_CONTAINER_NAME],
        None,
    )
    .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    .unwrap_or_else(|error| format!("(failed to read container logs: {error:#})"));
    let cause = last_error
        .map(|error| format!("{error:#}"))
        .unwrap_or_else(|| "no readiness check completed".to_owned());
    bail!(
        "local Keycloak {KEYCLOAK_CONTAINER_NAME} did not become ready within {}s: {cause}\ncontainer logs:\n{logs}",
        timeout.as_secs()
    );
}

fn check_ready(cluster_name: &str, ca_path: &Path) -> Result<()> {
    let discovery = fetch_json(KEYCLOAK_DISCOVERY_URL)?;
    validate_discovery(&discovery, KEYCLOAK_ISSUER)?;
    let node = local_cluster_server_node(cluster_name)?;
    let remote_ca = "/tmp/veoveo-local-keycloak-ca.pem";
    let destination = format!("{node}:{remote_ca}");
    status_checked(
        "docker",
        ["cp", path_str(ca_path)?, destination.as_str()],
        &[],
        None,
    )
    .context(
        "publishing the local Keycloak CA to the k3d server node for connectivity verification",
    )?;
    let internal_url = format!(
        "https://{KEYCLOAK_CONTAINER_NAME}:8443/realms/{KEYCLOAK_REALM_NAME}/.well-known/openid-configuration"
    );
    let internal_output = output_checked(
        "docker",
        [
            "exec",
            node.as_str(),
            "curl",
            "-sS",
            "--max-time",
            "5",
            "--cacert",
            remote_ca,
            internal_url.as_str(),
        ],
        None,
    )
    .with_context(|| format!("querying the local Keycloak internal endpoint {internal_url} from the k3d cluster network"))?;
    let internal: Value = serde_json::from_slice(&internal_output)
        .context("decoding the local Keycloak internal discovery document")?;
    validate_discovery(&internal, KEYCLOAK_ISSUER)?;
    let internal_prefix = format!("https://{KEYCLOAK_CONTAINER_NAME}:8443");
    for field in ["token_endpoint", "jwks_uri"] {
        let value = internal
            .get(field)
            .and_then(Value::as_str)
            .unwrap_or_default();
        ensure!(
            value.starts_with(&internal_prefix),
            "local Keycloak internal discovery `{field}` was {value}, expected the cluster-internal host {internal_prefix}"
        );
    }
    Ok(())
}

fn fetch_json(url: &str) -> Result<Value> {
    let output = output_checked("curl", ["-sS", "--max-time", "5", url], None)
        .with_context(|| format!("querying {url}"))?;
    serde_json::from_slice(&output).with_context(|| format!("decoding JSON from {url}"))
}

fn validate_discovery(discovery: &Value, expected_issuer: &str) -> Result<()> {
    let issuer = discovery.get("issuer").and_then(Value::as_str);
    ensure!(
        issuer == Some(expected_issuer),
        "local Keycloak discovery issuer was {issuer:?}, expected {expected_issuer:?}: {discovery}"
    );
    for field in ["authorization_endpoint", "token_endpoint", "jwks_uri"] {
        let value = discovery.get(field).and_then(Value::as_str);
        ensure!(
            value.is_some_and(|value| !value.is_empty()),
            "local Keycloak discovery omitted `{field}`: {discovery}"
        );
    }
    Ok(())
}

fn local_cluster_server_node(cluster_name: &str) -> Result<String> {
    let cluster_filter = format!("label=k3d.cluster={cluster_name}");
    let output = output_checked(
        "docker",
        [
            "ps",
            "--filter",
            cluster_filter.as_str(),
            "--filter",
            "label=k3d.role=server",
            "--format",
            "{{.Names}}",
        ],
        None,
    )?;
    String::from_utf8_lossy(&output)
        .lines()
        .next()
        .map(str::to_owned)
        .with_context(|| format!("no running k3d server node found for cluster {cluster_name}"))
}

pub(super) fn stop(_lock: &StateLock) -> Result<()> {
    if is_container_running(KEYCLOAK_CONTAINER_NAME)? {
        status_checked("docker", ["stop", KEYCLOAK_CONTAINER_NAME], &[], None)?;
    }
    Ok(())
}

pub(super) fn delete(_lock: &StateLock) -> Result<()> {
    if container_exists(KEYCLOAK_CONTAINER_NAME)? {
        status_checked("docker", ["rm", "-f", KEYCLOAK_CONTAINER_NAME], &[], None)?;
    }
    Ok(())
}
