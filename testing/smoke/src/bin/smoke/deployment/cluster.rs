//! Non-destructive readiness and recovery for profile-owned k3d clusters.

use std::{collections::BTreeMap, fmt, net::IpAddr};

use serde::Deserialize;

use super::K3dClusterSummary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClusterLifecycleDecision {
    Create,
    Start,
    Inspect,
}

pub(crate) fn cluster_lifecycle_decision(
    summary: Option<&K3dClusterSummary>,
) -> ClusterLifecycleDecision {
    match summary {
        None => ClusterLifecycleDecision::Create,
        Some(summary)
            if summary.servers_running == summary.servers_count
                && summary.agents_running == summary.agents_count =>
        {
            ClusterLifecycleDecision::Inspect
        }
        Some(_) => ClusterLifecycleDecision::Start,
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
            Self::K3dCounts {
                role,
                running,
                expected,
            } => write!(
                formatter,
                "k3d reports {running}/{expected} {role} nodes running",
                role = role.as_str()
            ),
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
    pub(crate) config: DockerConfig,
    #[serde(default)]
    pub(crate) state: DockerState,
    #[serde(default)]
    pub(crate) network_settings: DockerNetworkSettings,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct DockerConfig {
    #[serde(default)]
    pub(crate) labels: BTreeMap<String, String>,
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
    api_ready: bool,
    api_detail: impl Into<String>,
) -> ClusterHealth {
    let mut issues = Vec::new();
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
            .filter(|container| {
                container.config.labels.get("k3d.role").map(String::as_str) == Some(role.as_str())
            })
            .count() as u64;
        if found < expected {
            issues.push(ClusterIssue::MissingNodes {
                role,
                expected,
                found,
            });
        }
    }

    for container in containers {
        let name = container.name.trim_start_matches('/').to_owned();
        if container.state.restarting || container.state.status == "restarting" {
            issues.push(ClusterIssue::Restarting { name: name.clone() });
        } else if !container.state.running || container.state.status != "running" {
            issues.push(ClusterIssue::NotRunning {
                name: name.clone(),
                status: if container.state.status.is_empty() {
                    "unknown".to_owned()
                } else {
                    container.state.status.clone()
                },
            });
        }
        let Some(attachment) = container.network_settings.networks.get(network) else {
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
    if !api_ready {
        issues.push(ClusterIssue::ApiNotReady {
            detail: api_detail.into(),
        });
    }
    ClusterHealth { issues }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryDecision {
    Ready,
    Wait,
    StopStart,
    Fail,
}

/// Pure recovery policy. The caller owns timing and command execution.
pub(crate) fn recovery_decision(
    health: &ClusterHealth,
    spontaneous_deadline_reached: bool,
    recovery_attempted: bool,
) -> RecoveryDecision {
    if health.is_ready() {
        RecoveryDecision::Ready
    } else if !spontaneous_deadline_reached {
        RecoveryDecision::Wait
    } else if !recovery_attempted {
        RecoveryDecision::StopStart
    } else {
        RecoveryDecision::Fail
    }
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
        assert!(assess_cluster(&summary(), &[node()], "k3d-example", true, "ok").is_ready());
    }

    #[test]
    fn restarting_node_is_reported() {
        let mut container = node();
        container.state.restarting = true;
        container.state.status = "restarting".to_owned();
        let health = assess_cluster(&summary(), &[container], "k3d-example", true, "ok");
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
            .networks
            .get_mut("k3d-example")
            .expect("network fixture");
        attachment.endpoint_id.clear();
        attachment.ip_address = "not-an-ip".to_owned();
        let health = assess_cluster(&summary(), &[container], "k3d-example", true, "ok");
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
            recovery_decision(&health, false, false),
            RecoveryDecision::Wait,
            "an invalid node endpoint must not reach bootstrap apply"
        );
    }

    #[test]
    fn missing_ip_and_network_are_reported() {
        let mut container = node();
        container
            .network_settings
            .networks
            .get_mut("k3d-example")
            .expect("network fixture")
            .ip_address
            .clear();
        let health = assess_cluster(&summary(), &[container], "k3d-example", true, "ok");
        assert!(
            health
                .issues
                .iter()
                .any(|issue| matches!(issue, ClusterIssue::IpAbsent { .. }))
        );
        let health = assess_cluster(&summary(), &[node()], "k3d-missing", true, "ok");
        assert!(
            health
                .issues
                .iter()
                .any(|issue| matches!(issue, ClusterIssue::NetworkAbsent { .. }))
        );
    }

    #[test]
    fn api_not_ready_blocks_a_nominal_k3d_one_of_one() {
        let health = assess_cluster(&summary(), &[node()], "k3d-example", false, "EOF");
        assert!(
            health
                .issues
                .iter()
                .any(|issue| matches!(issue, ClusterIssue::ApiNotReady { .. }))
        );
    }

    #[test]
    fn policy_waits_then_restarts_once_then_fails() {
        let unhealthy = assess_cluster(&summary(), &[], "k3d-example", false, "timeout");
        assert_eq!(
            recovery_decision(&unhealthy, false, false),
            RecoveryDecision::Wait
        );
        assert_eq!(
            recovery_decision(&unhealthy, true, false),
            RecoveryDecision::StopStart
        );
        assert_eq!(
            recovery_decision(&unhealthy, false, true),
            RecoveryDecision::Wait
        );
        assert_eq!(
            recovery_decision(&unhealthy, true, true),
            RecoveryDecision::Fail
        );
    }

    #[test]
    fn spontaneous_recovery_and_stop_start_recovery_reach_ready() {
        let unhealthy = assess_cluster(&summary(), &[], "k3d-example", false, "503");
        assert_eq!(
            recovery_decision(&unhealthy, false, false),
            RecoveryDecision::Wait
        );
        let healthy = assess_cluster(&summary(), &[node()], "k3d-example", true, "ok");
        assert_eq!(
            recovery_decision(&healthy, false, false),
            RecoveryDecision::Ready
        );
        assert_eq!(
            recovery_decision(&healthy, true, true),
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
            ClusterLifecycleDecision::Start
        );
    }
}
