//! Non-destructive readiness and recovery for profile-owned k3d clusters.

use std::{collections::BTreeMap, fmt, net::IpAddr, time::Duration};

use serde::Deserialize;

use super::K3dClusterSummary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClusterLifecycleDecision {
    Create,
    Inspect,
}

pub(crate) fn cluster_lifecycle_decision(
    summary: Option<&K3dClusterSummary>,
) -> ClusterLifecycleDecision {
    match summary {
        None => ClusterLifecycleDecision::Create,
        Some(_) => ClusterLifecycleDecision::Inspect,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum K3dNodeRole {
    Server,
    Agent,
}

impl K3dNodeRole {
    fn as_str(self) -> &'static str {
        match self {
            Self::Server => "server",
            Self::Agent => "agent",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ClusterIssue {
    K3dInventoryFailed {
        detail: String,
    },
    DockerInspectionFailed {
        detail: String,
    },
    K3dCounts {
        role: K3dNodeRole,
        running: u64,
        expected: u64,
    },
    MissingNodes {
        role: K3dNodeRole,
        expected: u64,
        found: u64,
    },
    Restarting {
        name: String,
    },
    NotRunning {
        name: String,
        status: String,
    },
    NetworkAbsent {
        name: String,
        network: String,
    },
    EndpointIdAbsent {
        name: String,
        network: String,
    },
    IpAbsent {
        name: String,
        network: String,
    },
    IpInvalid {
        name: String,
        network: String,
        value: String,
    },
    ApiNotReady {
        detail: String,
    },
}

impl fmt::Display for ClusterIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::K3dInventoryFailed { detail } => {
                write!(formatter, "k3d cluster inventory failed: {detail}")
            }
            Self::K3dCounts {
                role,
                running,
                expected,
            } => write!(
                formatter,
                "k3d reports {running}/{expected} {role} nodes running",
                role = role.as_str()
            ),
            Self::DockerInspectionFailed { detail } => {
                write!(formatter, "Docker node inspection failed: {detail}")
            }
            Self::MissingNodes {
                role,
                expected,
                found,
            } => write!(
                formatter,
                "missing {role} nodes (expected {expected}, found {found})",
                role = role.as_str()
            ),
            Self::Restarting { name } => write!(formatter, "node {name} is restarting"),
            Self::NotRunning { name, status } => {
                write!(formatter, "node {name} is not running (state {status})")
            }
            Self::NetworkAbsent { name, network } => {
                write!(formatter, "node {name} has no {network} network attachment")
            }
            Self::EndpointIdAbsent { name, network } => write!(
                formatter,
                "node {name} has no EndpointID on network {network}"
            ),
            Self::IpAbsent { name, network } => {
                write!(formatter, "node {name} has no IP on network {network}")
            }
            Self::IpInvalid {
                name,
                network,
                value,
            } => write!(
                formatter,
                "node {name} has invalid IP {value:?} on network {network}"
            ),
            Self::ApiNotReady { detail } => {
                write!(formatter, "Kubernetes API is not ready: {detail}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClusterHealth {
    pub(crate) issues: Vec<ClusterIssue>,
    pub(crate) all_expected_nodes_stopped: bool,
}

impl ClusterHealth {
    pub(crate) fn is_ready(&self) -> bool {
        self.issues.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct DockerContainer {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) config: Option<DockerConfig>,
    #[serde(default)]
    pub(crate) state: Option<DockerState>,
    #[serde(default)]
    pub(crate) network_settings: Option<DockerNetworkSettings>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct DockerConfig {
    #[serde(default)]
    pub(crate) labels: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct DockerState {
    #[serde(default)]
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) running: bool,
    #[serde(default)]
    pub(crate) restarting: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct DockerNetworkSettings {
    #[serde(default)]
    pub(crate) networks: BTreeMap<String, DockerNetworkAttachment>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct DockerNetworkAttachment {
    #[serde(rename = "EndpointID", default)]
    pub(crate) endpoint_id: String,
    #[serde(rename = "IPAddress", default)]
    pub(crate) ip_address: String,
}

pub(crate) fn assess_cluster(
    summary: &K3dClusterSummary,
    containers: &[DockerContainer],
    network: &str,
    api: ApiHealth,
) -> ClusterHealth {
    let mut issues = Vec::new();
    let node_containers = containers
        .iter()
        .filter(|container| node_role(container).is_some())
        .collect::<Vec<_>>();
    for (role, expected) in [
        (K3dNodeRole::Server, summary.servers_count),
        (K3dNodeRole::Agent, summary.agents_count),
    ] {
        let running = match role {
            K3dNodeRole::Server => summary.servers_running,
            K3dNodeRole::Agent => summary.agents_running,
        };
        if running != expected {
            issues.push(ClusterIssue::K3dCounts {
                role,
                running,
                expected,
            });
        }
        let found = containers
            .iter()
            .filter(|container| node_role(container) == Some(role))
            .count() as u64;
        if found < expected {
            issues.push(ClusterIssue::MissingNodes {
                role,
                expected,
                found,
            });
        }
    }

    for container in node_containers.iter().copied() {
        let name = container.name.trim_start_matches('/').to_owned();
        let Some(state) = &container.state else {
            issues.push(ClusterIssue::NotRunning {
                name: name.clone(),
                status: "unknown".to_owned(),
            });
            continue;
        };
        if state.restarting || state.status == "restarting" {
            issues.push(ClusterIssue::Restarting { name: name.clone() });
        } else if !state.running || state.status != "running" {
            issues.push(ClusterIssue::NotRunning {
                name: name.clone(),
                status: if state.status.is_empty() {
                    "unknown".to_owned()
                } else {
                    state.status.clone()
                },
            });
        }
        let Some(network_settings) = &container.network_settings else {
            issues.push(ClusterIssue::NetworkAbsent {
                name,
                network: network.to_owned(),
            });
            continue;
        };
        let Some(attachment) = network_settings.networks.get(network) else {
            issues.push(ClusterIssue::NetworkAbsent {
                name,
                network: network.to_owned(),
            });
            continue;
        };
        if attachment.endpoint_id.trim().is_empty() {
            issues.push(ClusterIssue::EndpointIdAbsent {
                name: name.clone(),
                network: network.to_owned(),
            });
        }
        if attachment.ip_address.trim().is_empty() {
            issues.push(ClusterIssue::IpAbsent {
                name,
                network: network.to_owned(),
            });
        } else if attachment
            .ip_address
            .parse::<IpAddr>()
            .map_or(true, |address| address.is_unspecified())
        {
            issues.push(ClusterIssue::IpInvalid {
                name,
                network: network.to_owned(),
                value: attachment.ip_address.clone(),
            });
        }
    }
    if let ApiHealth::NotReady(detail) = api {
        issues.push(ClusterIssue::ApiNotReady { detail });
    }
    let expected_nodes = summary.servers_count + summary.agents_count;
    let all_expected_nodes_stopped = expected_nodes > 0
        && node_containers.len() as u64 >= expected_nodes
        && node_containers.iter().all(|container| {
            container
                .state
                .as_ref()
                .is_some_and(|state| !state.running && !state.restarting)
        });
    ClusterHealth {
        issues,
        all_expected_nodes_stopped,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ApiHealth {
    Ready,
    NotReady(String),
}

fn node_role(container: &DockerContainer) -> Option<K3dNodeRole> {
    let role = container
        .config
        .as_ref()?
        .labels
        .as_ref()?
        .get("k3d.role")?;
    match role.as_str() {
        "server" => Some(K3dNodeRole::Server),
        "agent" => Some(K3dNodeRole::Agent),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryDecision {
    Ready,
    Wait,
    Start,
    StopStart,
    Fail,
}

/// Pure recovery policy. The caller owns timing and command execution.
pub(crate) fn recovery_decision(
    health: &ClusterHealth,
    spontaneous_deadline_reached: bool,
    recovery_attempted: bool,
    start_attempted: bool,
) -> RecoveryDecision {
    if health.is_ready() {
        RecoveryDecision::Ready
    } else if health.all_expected_nodes_stopped && !start_attempted && !recovery_attempted {
        RecoveryDecision::Start
    } else if !spontaneous_deadline_reached {
        RecoveryDecision::Wait
    } else if !recovery_attempted {
        RecoveryDecision::StopStart
    } else {
        RecoveryDecision::Fail
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClusterPresence {
    Absent,
    Existing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LifecycleAction {
    Create,
    Start,
    Stop,
    ApplyBootstrap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LifecycleFailure {
    pub(crate) detail: String,
    pub(crate) manual_recovery: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecoveryConfig {
    pub(crate) spontaneous_window: Duration,
    pub(crate) restart_window: Duration,
    pub(crate) manual_recovery: String,
}

impl fmt::Display for LifecycleFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}\n\n{}", self.detail, self.manual_recovery)
    }
}

pub(crate) fn orchestrate_cluster<P, A, S, C, D>(
    presence: ClusterPresence,
    mut probe: P,
    mut action: A,
    mut sleep: S,
    mut clock: C,
    mut diagnose: D,
    config: RecoveryConfig,
) -> Result<(), LifecycleFailure>
where
    P: FnMut() -> ClusterHealth,
    A: FnMut(LifecycleAction) -> Result<(), String>,
    S: FnMut(),
    C: FnMut() -> Duration,
    D: FnMut(&ClusterHealth),
{
    if presence == ClusterPresence::Absent
        && let Err(detail) = action(LifecycleAction::Create)
    {
        return Err(lifecycle_failure(
            format!("k3d cluster creation failed: {detail}"),
            config.manual_recovery,
        ));
    }
    let mut recovery_attempted = false;
    let mut start_attempted = false;
    let mut deadline = clock() + config.spontaneous_window;
    loop {
        let health = probe();
        match recovery_decision(
            &health,
            clock() >= deadline,
            recovery_attempted,
            start_attempted,
        ) {
            RecoveryDecision::Ready => {
                if let Err(detail) = action(LifecycleAction::ApplyBootstrap) {
                    return Err(lifecycle_failure(
                        format!(
                            "kubectl bootstrap apply failed: {detail}; observed state: {}",
                            format_health(&health)
                        ),
                        config.manual_recovery,
                    ));
                }
                return Ok(());
            }
            RecoveryDecision::Wait => {
                diagnose(&health);
                sleep();
            }
            RecoveryDecision::Start => {
                if let Err(detail) = action(LifecycleAction::Start) {
                    return Err(lifecycle_failure(
                        format!(
                            "k3d cluster start failed while nodes were stopped: {detail}; observed state: {}",
                            format_health(&health)
                        ),
                        config.manual_recovery,
                    ));
                }
                start_attempted = true;
                deadline = clock() + config.restart_window;
            }
            RecoveryDecision::StopStart => {
                diagnose(&health);
                if let Err(detail) = action(LifecycleAction::Stop) {
                    return Err(lifecycle_failure(
                        format!(
                            "k3d cluster stop failed during non-destructive recovery: {detail}; observed state: {}",
                            format_health(&health)
                        ),
                        config.manual_recovery,
                    ));
                }
                if let Err(detail) = action(LifecycleAction::Start) {
                    return Err(lifecycle_failure(
                        format!(
                            "k3d cluster start failed during non-destructive recovery: {detail}; observed state: {}",
                            format_health(&health)
                        ),
                        config.manual_recovery,
                    ));
                }
                recovery_attempted = true;
                deadline = clock() + config.restart_window;
            }
            RecoveryDecision::Fail => {
                return Err(lifecycle_failure(
                    format!(
                        "k3d cluster remains unhealthy after spontaneous recovery and one non-destructive stop/start: {}",
                        format_health(&health)
                    ),
                    config.manual_recovery,
                ));
            }
        }
    }
}

fn lifecycle_failure(detail: String, manual_recovery: String) -> LifecycleFailure {
    LifecycleFailure {
        detail,
        manual_recovery,
    }
}

fn format_health(health: &ClusterHealth) -> String {
    health
        .issues
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary() -> K3dClusterSummary {
        K3dClusterSummary {
            name: "example".to_owned(),
            servers_running: 1,
            servers_count: 1,
            agents_running: 0,
            agents_count: 0,
        }
    }

    fn node() -> DockerContainer {
        serde_json::from_value(serde_json::json!({
            "Name": "/k3d-example-server-0",
            "Config": {"Labels": {"k3d.role": "server"}},
            "State": {"Status": "running", "Running": true, "Restarting": false},
            "NetworkSettings": {"Networks": {
                "k3d-example": {"EndpointID": "endpoint", "IPAddress": "172.20.0.2"}
            }}
        }))
        .expect("docker fixture")
    }

    #[test]
    fn healthy_nodes_and_api_are_ready() {
        assert!(assess_cluster(&summary(), &[node()], "k3d-example", ApiHealth::Ready).is_ready());
    }

    #[test]
    fn restarting_node_is_reported() {
        let mut container = node();
        let state = container.state.as_mut().expect("state fixture");
        state.restarting = true;
        state.status = "restarting".to_owned();
        let health = assess_cluster(&summary(), &[container], "k3d-example", ApiHealth::Ready);
        assert!(
            health
                .issues
                .iter()
                .any(|issue| matches!(issue, ClusterIssue::Restarting { .. }))
        );
    }

    #[test]
    fn lost_endpoint_and_invalid_ip_are_reported() {
        let mut container = node();
        let attachment = container
            .network_settings
            .as_mut()
            .expect("network settings")
            .networks
            .get_mut("k3d-example")
            .expect("network fixture");
        attachment.endpoint_id.clear();
        attachment.ip_address = "not-an-ip".to_owned();
        let health = assess_cluster(&summary(), &[container], "k3d-example", ApiHealth::Ready);
        assert!(
            health
                .issues
                .iter()
                .any(|issue| matches!(issue, ClusterIssue::EndpointIdAbsent { .. }))
        );
        assert!(
            health
                .issues
                .iter()
                .any(|issue| matches!(issue, ClusterIssue::IpInvalid { .. }))
        );
        assert_eq!(
            recovery_decision(&health, false, false, false),
            RecoveryDecision::Wait,
            "an invalid node endpoint must not reach bootstrap apply"
        );
    }

    #[test]
    fn missing_ip_and_network_are_reported() {
        let mut container = node();
        container
            .network_settings
            .as_mut()
            .expect("network settings")
            .networks
            .get_mut("k3d-example")
            .expect("network fixture")
            .ip_address
            .clear();
        let health = assess_cluster(&summary(), &[container], "k3d-example", ApiHealth::Ready);
        assert!(
            health
                .issues
                .iter()
                .any(|issue| matches!(issue, ClusterIssue::IpAbsent { .. }))
        );
        let health = assess_cluster(&summary(), &[node()], "k3d-missing", ApiHealth::Ready);
        assert!(
            health
                .issues
                .iter()
                .any(|issue| matches!(issue, ClusterIssue::NetworkAbsent { .. }))
        );
    }

    #[test]
    fn helper_containers_and_null_docker_sections_do_not_block_node_health() {
        let helper: DockerContainer = serde_json::from_value(serde_json::json!({
            "Name": "/k3d-example-serverlb",
            "Config": {"Labels": {"k3d.role": "serverlb"}},
            "State": {"Status": "exited", "Running": false},
            "NetworkSettings": null
        }))
        .expect("helper fixture");
        let health = assess_cluster(
            &summary(),
            &[node(), helper],
            "k3d-example",
            ApiHealth::Ready,
        );
        assert!(health.is_ready());
        let null_sections: DockerContainer = serde_json::from_value(serde_json::json!({
            "Name": "/k3d-example-tools",
            "Config": null,
            "State": null,
            "NetworkSettings": null
        }))
        .expect("null Docker sections are valid inspect output");
        assert!(null_sections.config.is_none());
        assert!(null_sections.network_settings.is_none());
    }

    #[test]
    fn api_not_ready_blocks_a_nominal_k3d_one_of_one() {
        let health = assess_cluster(
            &summary(),
            &[node()],
            "k3d-example",
            ApiHealth::NotReady("EOF".to_owned()),
        );
        assert!(
            health
                .issues
                .iter()
                .any(|issue| matches!(issue, ClusterIssue::ApiNotReady { .. }))
        );
    }

    #[test]
    fn policy_waits_then_restarts_once_then_fails() {
        let unhealthy = assess_cluster(
            &summary(),
            &[],
            "k3d-example",
            ApiHealth::NotReady("timeout".to_owned()),
        );
        assert_eq!(
            recovery_decision(&unhealthy, false, false, false),
            RecoveryDecision::Wait
        );
        assert_eq!(
            recovery_decision(&unhealthy, true, false, false),
            RecoveryDecision::StopStart
        );
        assert_eq!(
            recovery_decision(&unhealthy, false, true, true),
            RecoveryDecision::Wait
        );
        assert_eq!(
            recovery_decision(&unhealthy, true, true, true),
            RecoveryDecision::Fail
        );
    }

    #[test]
    fn fully_stopped_nodes_use_supported_start_but_restarting_nodes_wait() {
        let mut container = node();
        let state = container.state.as_mut().expect("state fixture");
        state.running = false;
        state.status = "exited".to_owned();
        let stopped = assess_cluster(
            &summary(),
            &[container],
            "k3d-example",
            ApiHealth::NotReady("connection refused".to_owned()),
        );
        assert_eq!(
            recovery_decision(&stopped, false, false, false),
            RecoveryDecision::Start
        );
        let restarting = restarting();
        assert_eq!(
            recovery_decision(&restarting, false, false, false),
            RecoveryDecision::Wait
        );
    }

    #[test]
    fn spontaneous_recovery_and_stop_start_recovery_reach_ready() {
        let unhealthy = assess_cluster(
            &summary(),
            &[],
            "k3d-example",
            ApiHealth::NotReady("503".to_owned()),
        );
        assert_eq!(
            recovery_decision(&unhealthy, false, false, false),
            RecoveryDecision::Wait
        );
        let healthy = assess_cluster(&summary(), &[node()], "k3d-example", ApiHealth::Ready);
        assert_eq!(
            recovery_decision(&healthy, false, false, false),
            RecoveryDecision::Ready
        );
        assert_eq!(
            recovery_decision(&healthy, true, true, true),
            RecoveryDecision::Ready
        );
    }

    #[test]
    fn absent_cluster_uses_create_path_and_healthy_cluster_is_not_restarted() {
        assert_eq!(
            cluster_lifecycle_decision(None),
            ClusterLifecycleDecision::Create
        );
        assert_eq!(
            cluster_lifecycle_decision(Some(&summary())),
            ClusterLifecycleDecision::Inspect
        );
        let mut starting = summary();
        starting.servers_running = 0;
        assert_eq!(
            cluster_lifecycle_decision(Some(&starting)),
            ClusterLifecycleDecision::Inspect
        );
    }

    fn healthy() -> ClusterHealth {
        assess_cluster(&summary(), &[node()], "k3d-example", ApiHealth::Ready)
    }

    fn restarting() -> ClusterHealth {
        let mut container = node();
        let state = container.state.as_mut().expect("state fixture");
        state.running = false;
        state.restarting = true;
        state.status = "restarting".to_owned();
        assess_cluster(
            &summary(),
            &[container],
            "k3d-example",
            ApiHealth::NotReady("EOF".to_owned()),
        )
    }

    fn endpoint_lost() -> ClusterHealth {
        let mut container = node();
        container
            .network_settings
            .as_mut()
            .expect("network settings")
            .networks
            .get_mut("k3d-example")
            .expect("network fixture")
            .endpoint_id
            .clear();
        assess_cluster(&summary(), &[container], "k3d-example", ApiHealth::Ready)
    }

    fn run_script(
        presence: ClusterPresence,
        probes: Vec<ClusterHealth>,
    ) -> (Vec<LifecycleAction>, Result<(), LifecycleFailure>) {
        use std::cell::Cell;

        let mut probes = probes.into_iter();
        let mut events = Vec::new();
        let now = Cell::new(Duration::ZERO);
        let result = orchestrate_cluster(
            presence,
            || probes.next().expect("probe fixture"),
            |action| {
                events.push(action);
                Ok(())
            },
            || now.set(now.get() + Duration::from_secs(2)),
            || now.get(),
            |_| {},
            RecoveryConfig {
                spontaneous_window: Duration::from_secs(1),
                restart_window: Duration::from_secs(1),
                manual_recovery: "cargo xtask smoke profile-cluster-delete --profile test.json\nWARNING: PVC data may be lost".to_owned(),
            },
        );
        (events, result)
    }

    #[test]
    fn lifecycle_healthy_does_not_restart_and_applies_once() {
        let (events, result) = run_script(ClusterPresence::Existing, vec![healthy()]);
        assert_eq!(events, vec![LifecycleAction::ApplyBootstrap]);
        assert!(result.is_ok());
    }

    #[test]
    fn lifecycle_absent_creates_then_applies_after_ready() {
        let (events, result) = run_script(ClusterPresence::Absent, vec![healthy()]);
        assert_eq!(
            events,
            vec![LifecycleAction::Create, LifecycleAction::ApplyBootstrap]
        );
        assert!(result.is_ok());
    }

    #[test]
    fn lifecycle_restarting_waits_then_stop_starts_once_before_apply() {
        let (events, result) = run_script(
            ClusterPresence::Existing,
            vec![restarting(), restarting(), healthy()],
        );
        assert_eq!(
            events,
            vec![
                LifecycleAction::Stop,
                LifecycleAction::Start,
                LifecycleAction::ApplyBootstrap
            ]
        );
        assert!(result.is_ok());
    }

    #[test]
    fn lifecycle_lost_endpoint_waits_then_stop_starts_once_before_apply() {
        let (events, result) = run_script(
            ClusterPresence::Existing,
            vec![endpoint_lost(), endpoint_lost(), healthy()],
        );
        assert_eq!(
            events,
            vec![
                LifecycleAction::Stop,
                LifecycleAction::Start,
                LifecycleAction::ApplyBootstrap
            ]
        );
        assert!(result.is_ok());
    }

    #[test]
    fn lifecycle_spontaneous_recovery_never_stop_starts() {
        let (events, result) = run_script(ClusterPresence::Existing, vec![restarting(), healthy()]);
        assert_eq!(events, vec![LifecycleAction::ApplyBootstrap]);
        assert!(result.is_ok());
    }

    #[test]
    fn lifecycle_failure_never_applies_and_preserves_manual_warning() {
        let (events, result) = run_script(
            ClusterPresence::Existing,
            vec![restarting(), restarting(), restarting(), restarting()],
        );
        assert_eq!(events, vec![LifecycleAction::Stop, LifecycleAction::Start]);
        let failure = result.expect_err("unhealthy fixture must fail");
        assert!(failure.to_string().contains("profile-cluster-delete"));
        assert!(failure.to_string().contains("PVC"));
        assert!(!events.contains(&LifecycleAction::ApplyBootstrap));
    }

    #[test]
    fn lifecycle_stop_failure_keeps_state_and_manual_recovery_diagnostics() {
        use std::cell::Cell;

        let now = Cell::new(Duration::ZERO);
        let mut probes = vec![restarting(), restarting()].into_iter();
        let mut events = Vec::new();
        let result = orchestrate_cluster(
            ClusterPresence::Existing,
            || probes.next().expect("probe fixture"),
            |action| {
                events.push(action);
                if action == LifecycleAction::Stop {
                    Err("stop unavailable".to_owned())
                } else {
                    Ok(())
                }
            },
            || now.set(now.get() + Duration::from_secs(2)),
            || now.get(),
            |_| {},
            RecoveryConfig {
                spontaneous_window: Duration::from_secs(1),
                restart_window: Duration::from_secs(1),
                manual_recovery:
                    "profile-cluster-delete --profile test.json\nWARNING: PVC data may be lost"
                        .to_owned(),
            },
        );
        assert_eq!(events, vec![LifecycleAction::Stop]);
        let failure = result.expect_err("stop failure must abort");
        assert!(failure.to_string().contains("stop unavailable"));
        assert!(failure.to_string().contains("profile-cluster-delete"));
        assert!(failure.to_string().contains("PVC"));
    }
}
