use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};

use serde_json::Value;
use veoveo_deploy_contract::{
    DeploymentProfile, GatewayActivationSpec, GeneratedPublicFile, GeneratedPublicFileKind,
    InstallationPreset, KubernetesTarget, LoadedProfile, LocalClusterSpec, PROFILE_SCHEMA,
    PlatformSelection, RegistryReference, RegistryTransport, ResourceSet,
};

use super::{
    CERT_MOUNT_DESTINATION, KEY_MOUNT_DESTINATION, KEYCLOAK_ENV_VARS, KEYCLOAK_IMAGE,
    LOCAL_KEYCLOAK_CONFIG_DIGEST_LABEL, REALM_MOUNT_DESTINATION,
    ensure_generated_public_files_with, profile_requires_local_keycloak, startup_args,
};
use super::{
    container::{
        ContainerInspect, ContainerInspectConfig, ContainerInspectMount,
        ContainerInspectNetworkSettings, FingerprintInputs, config_fingerprint, container_is_valid,
    },
    tls::{
        StateLock, active_generation, ensure_generation, generate_material,
        restrict_key_permissions, state_dir,
        test_support::{current_pointer, generation_directory},
        validate_generation,
    },
};
use crate::deployment::{prepare_gateway_activation, validate_gateway_activation};

struct Fixture {
    _dir: tempfile::TempDir,
    profile: LoadedProfile,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("create tempdir");
    let root = dir.path().canonicalize().expect("canonicalize tempdir");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::copy(
        repository.join("showcase/sumo/deploy/gateway.json"),
        root.join("gateway.json"),
    )
    .expect("copy gateway fixture");
    fs::copy(
        repository.join("showcase/sumo/deploy/jwks.json"),
        root.join("jwks.json"),
    )
    .expect("copy JWKS fixture");

    let definition = DeploymentProfile {
        schema_version: PROFILE_SCHEMA.to_owned(),
        name: "keycloak-test".to_owned(),
        registry: RegistryReference {
            push_address: "127.0.0.1:5001".to_owned(),
            pull_address: "127.0.0.1:5001".to_owned(),
            transport: RegistryTransport::InsecureHttp,
            local_config: None,
        },
        sources: Vec::new(),
        kubernetes: KubernetesTarget {
            context: "test".to_owned(),
            local_cluster: Some(LocalClusterSpec {
                name: format!("keycloak-test-{}", std::process::id()),
                config: PathBuf::from("cluster.yaml"),
                node_bootstrap_manifests: Vec::new(),
            }),
        },
        namespace: "veoveo".to_owned(),
        resources: ResourceSet::default(),
        gateway_activation: Some(GatewayActivationSpec {
            config_map_name_prefix: "veoveo-gateway".to_owned(),
            control_plane_key: "gateway.json".to_owned(),
            control_plane: PathBuf::from("gateway.json"),
            public_files: BTreeMap::from([("jwks.json".to_owned(), PathBuf::from("jwks.json"))]),
            generated_public_files: BTreeMap::from([(
                "ca.pem".to_owned(),
                GeneratedPublicFile {
                    kind: GeneratedPublicFileKind::LocalKeycloakCa,
                },
            )]),
            confidential_secret: "installation-secrets".to_owned(),
            required_secret_keys: BTreeSet::from(["oidc-client-secret".to_owned()]),
        }),
        platform: PlatformSelection {
            installation_preset: InstallationPreset::Custom,
            components: BTreeSet::new(),
            mcp_servers: BTreeSet::new(),
            artifact_audiences: BTreeSet::new(),
            external_workloads: BTreeSet::new(),
            gpu_scheduling: None,
        },
        gateway_requirements: Vec::new(),
        wait_for_deployments: Vec::new(),
    };
    let profile = LoadedProfile {
        definition,
        path: root.join("deployment.json"),
        directory: root.clone(),
        repository: root,
    };
    Fixture { _dir: dir, profile }
}

#[test]
fn structural_validation_does_not_require_or_create_runtime_state() {
    let fixture = fixture();
    assert!(!state_dir(&fixture.profile.repository).exists());
    validate_gateway_activation(&fixture.profile).expect("validate generated file structure");
    assert!(profile_requires_local_keycloak(&fixture.profile).unwrap());
    assert!(!state_dir(&fixture.profile.repository).exists());
}

#[test]
fn generated_ca_is_held_consistent_through_activation_preparation() {
    let fixture = fixture();
    let generated =
        ensure_generated_public_files_with(&fixture.profile, |_, _| Ok(())).expect("generate CA");
    let activation = prepare_gateway_activation(&fixture.profile, &generated)
        .expect("prepare activation")
        .expect("activation exists");
    let generation = active_generation(&fixture.profile.repository)
        .unwrap()
        .expect("generation exists");
    assert_eq!(
        activation.data["ca.pem"],
        fs::read_to_string(generation.ca_path).unwrap()
    );
}

#[test]
fn generated_public_files_reconcile_keycloak_before_returning() {
    use std::cell::Cell;

    let fixture = fixture();
    let reconciled = Cell::new(0_u8);
    let generated = ensure_generated_public_files_with(&fixture.profile, |profile, _lock| {
        assert!(std::ptr::eq(profile, &fixture.profile));
        reconciled.set(reconciled.get() + 1);
        Ok(())
    })
    .expect("materialize and reconcile local Keycloak");

    assert_eq!(reconciled.get(), 1);
    assert!(generated.path("ca.pem").is_some());
}

#[test]
fn invalid_and_traversal_current_pointers_are_rejected() {
    let fixture = fixture();
    fs::create_dir_all(state_dir(&fixture.profile.repository)).unwrap();
    for pointer in ["../escape", "A".repeat(64).as_str(), "abc", "/tmp/escape"] {
        fs::write(current_pointer(&fixture.profile.repository), pointer).unwrap();
        assert!(
            active_generation(&fixture.profile.repository)
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn valid_material_under_the_wrong_digest_name_is_rejected() {
    let fixture = fixture();
    let lock = StateLock::acquire().unwrap();
    let generation = ensure_generation(&fixture.profile.repository, &lock).unwrap();
    let wrong_digest = "a".repeat(64);
    let wrong_dir = generation_directory(&fixture.profile.repository, &wrong_digest);
    fs::rename(generation.cert_path.parent().unwrap(), &wrong_dir).unwrap();
    fs::write(current_pointer(&fixture.profile.repository), &wrong_digest).unwrap();
    assert!(
        active_generation(&fixture.profile.repository)
            .unwrap()
            .is_none()
    );
}

#[test]
fn generated_material_is_cryptographically_valid() {
    let fixture = fixture();
    let lock = StateLock::acquire().unwrap();
    let generation = ensure_generation(&fixture.profile.repository, &lock).unwrap();
    validate_generation(
        &generation.ca_path,
        &generation.cert_path,
        &generation.key_path,
    )
    .unwrap();
}

fn standalone_material(directory: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let ca = directory.join("ca.pem");
    let cert = directory.join("tls.crt");
    let key = directory.join("tls.key");
    generate_material(&ca, &cert, &key).unwrap();
    restrict_key_permissions(&key).unwrap();
    (ca, cert, key)
}

#[test]
fn corrupt_or_incomplete_tls_material_is_rejected() {
    for corrupt in ["ca", "cert", "key"] {
        let dir = tempfile::tempdir().unwrap();
        let (ca, cert, key) = standalone_material(dir.path());
        let path = match corrupt {
            "ca" => &ca,
            "cert" => &cert,
            _ => &key,
        };
        fs::write(path, b"corrupt").unwrap();
        assert!(validate_generation(&ca, &cert, &key).is_err());
    }
    let dir = tempfile::tempdir().unwrap();
    let (ca, cert, key) = standalone_material(dir.path());
    fs::remove_file(&key).unwrap();
    assert!(validate_generation(&ca, &cert, &key).is_err());
}

#[test]
fn mismatched_private_key_and_signing_ca_are_rejected() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let (ca, cert, key) = standalone_material(first.path());
    let (other_ca, _, other_key) = standalone_material(second.path());
    fs::copy(&other_key, &key).unwrap();
    assert!(validate_generation(&ca, &cert, &key).is_err());

    let (ca, cert, key) = standalone_material(first.path());
    fs::copy(&other_ca, &ca).unwrap();
    assert!(validate_generation(&ca, &cert, &key).is_err());
}

#[test]
fn ensure_replaces_a_corrupt_active_generation() {
    let fixture = fixture();
    let lock = StateLock::acquire().unwrap();
    let first = ensure_generation(&fixture.profile.repository, &lock).unwrap();
    fs::write(&first.cert_path, b"corrupt").unwrap();
    let second = ensure_generation(&fixture.profile.repository, &lock).unwrap();
    assert_ne!(first.digest, second.digest);
    validate_generation(&second.ca_path, &second.cert_path, &second.key_path).unwrap();
}

#[test]
#[cfg(unix)]
fn group_readable_private_key_is_rejected() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let (ca, cert, key) = standalone_material(dir.path());
    fs::set_permissions(&key, fs::Permissions::from_mode(0o640)).unwrap();
    assert!(validate_generation(&ca, &cert, &key).is_err());
}

#[test]
fn concurrent_ensures_publish_one_reusable_generation() {
    let dir = tempfile::tempdir().unwrap();
    let repository = dir.path().to_path_buf();
    let lock_path = dir.path().join("shared.lock");
    let handles = (0..8)
        .map(|_| {
            let repository = repository.clone();
            let lock_path = lock_path.clone();
            thread::spawn(move || {
                let lock = StateLock::acquire_at(&lock_path).unwrap();
                ensure_generation(&repository, &lock).unwrap().digest
            })
        })
        .collect::<Vec<_>>();
    let digests = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(digests.len(), 1);
}

#[test]
fn lock_process_helper() {
    let Ok(signal) = std::env::var("VEOVEO_KEYCLOAK_LOCK_HELPER_SIGNAL") else {
        return;
    };
    let _lock = if std::env::var_os("VEOVEO_KEYCLOAK_LOCK_HELPER_GLOBAL").is_some() {
        StateLock::acquire().unwrap()
    } else {
        let lock_path = std::env::var("VEOVEO_KEYCLOAK_LOCK_HELPER_PATH").unwrap();
        StateLock::acquire_at(Path::new(&lock_path)).unwrap()
    };
    fs::write(&signal, b"acquired").unwrap();
}

#[test]
fn state_lock_serializes_independent_processes() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("interprocess.lock");
    let signal = dir.path().join("child-acquired");
    let parent_lock = StateLock::acquire_at(&lock_path).unwrap();
    let module = module_path!()
        .split_once("::")
        .map_or(module_path!(), |(_, module)| module);
    let exact_test = format!("{module}::lock_process_helper");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &exact_test, "--nocapture"])
        .env("VEOVEO_KEYCLOAK_LOCK_HELPER_PATH", &lock_path)
        .env("VEOVEO_KEYCLOAK_LOCK_HELPER_SIGNAL", &signal)
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_millis(250));
    assert!(
        !signal.exists(),
        "child acquired a lock still held by parent"
    );
    drop(parent_lock);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !signal.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(signal.exists(), "child never acquired the released lock");
    assert!(child.wait().unwrap().success());
}

#[test]
fn global_lock_serializes_profiles_with_different_cluster_names() {
    let mut first = fixture();
    let mut second = fixture();
    first
        .profile
        .definition
        .kubernetes
        .local_cluster
        .as_mut()
        .unwrap()
        .name = "first-cluster".to_owned();
    second
        .profile
        .definition
        .kubernetes
        .local_cluster
        .as_mut()
        .unwrap()
        .name = "second-cluster".to_owned();
    assert_ne!(
        first
            .profile
            .definition
            .kubernetes
            .local_cluster
            .as_ref()
            .unwrap()
            .name,
        second
            .profile
            .definition
            .kubernetes
            .local_cluster
            .as_ref()
            .unwrap()
            .name
    );

    let dir = tempfile::tempdir().unwrap();
    let signal = dir.path().join("global-child-acquired");
    let parent_lock = StateLock::acquire().unwrap();
    let module = module_path!()
        .split_once("::")
        .map_or(module_path!(), |(_, module)| module);
    let exact_test = format!("{module}::lock_process_helper");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &exact_test, "--nocapture"])
        .env("VEOVEO_KEYCLOAK_LOCK_HELPER_GLOBAL", "1")
        .env("VEOVEO_KEYCLOAK_LOCK_HELPER_SIGNAL", &signal)
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_millis(250));
    assert!(
        !signal.exists(),
        "a differently named profile bypassed the global lock"
    );
    drop(parent_lock);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !signal.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        signal.exists(),
        "child never acquired the released global lock"
    );
    assert!(child.wait().unwrap().success());
}

#[test]
fn fingerprint_is_deterministic_and_tracks_configuration() {
    let args = startup_args();
    let mounts = [
        REALM_MOUNT_DESTINATION,
        CERT_MOUNT_DESTINATION,
        KEY_MOUNT_DESTINATION,
    ];
    let fingerprint = |realm: &[u8], cert: &[u8], network: &str| {
        config_fingerprint(&FingerprintInputs {
            image: KEYCLOAK_IMAGE,
            realm_bytes: realm,
            startup_args: &args,
            env_vars: KEYCLOAK_ENV_VARS,
            server_cert_pem: cert,
            active_generation_digest: "aabbccdd",
            mount_destinations: &mounts,
            network,
        })
    };
    let realm = b"{\"realm\":\"veoveo-local\"}";
    let cert = b"certificate";
    let baseline = fingerprint(realm, cert, "k3d-sumo");
    assert_eq!(baseline, fingerprint(realm, cert, "k3d-sumo"));
    assert_ne!(baseline, fingerprint(b"changed", cert, "k3d-sumo"));
    assert_ne!(baseline, fingerprint(realm, b"changed", "k3d-sumo"));
    assert_ne!(baseline, fingerprint(realm, cert, "k3d-other"));
}

fn sample_inspect(fingerprint: Option<&str>, network: &str) -> ContainerInspect {
    ContainerInspect {
        config: ContainerInspectConfig {
            image: KEYCLOAK_IMAGE.to_owned(),
            labels: fingerprint.map(|value| {
                BTreeMap::from([(
                    LOCAL_KEYCLOAK_CONFIG_DIGEST_LABEL.to_owned(),
                    value.to_owned(),
                )])
            }),
        },
        mounts: vec![
            ContainerInspectMount {
                source: "/realm".to_owned(),
                destination: REALM_MOUNT_DESTINATION.to_owned(),
            },
            ContainerInspectMount {
                source: "/cert".to_owned(),
                destination: CERT_MOUNT_DESTINATION.to_owned(),
            },
            ContainerInspectMount {
                source: "/key".to_owned(),
                destination: KEY_MOUNT_DESTINATION.to_owned(),
            },
        ],
        network_settings: ContainerInspectNetworkSettings {
            networks: BTreeMap::from([(network.to_owned(), Value::Null)]),
        },
    }
}

#[test]
fn container_reuse_requires_label_network_and_mounts() {
    let mounts = [
        (Path::new("/realm"), REALM_MOUNT_DESTINATION),
        (Path::new("/cert"), CERT_MOUNT_DESTINATION),
        (Path::new("/key"), KEY_MOUNT_DESTINATION),
    ];
    assert!(
        container_is_valid(
            &sample_inspect(Some("expected"), "network"),
            "network",
            &mounts,
            "expected"
        )
        .unwrap()
    );
    assert!(
        !container_is_valid(
            &sample_inspect(None, "network"),
            "network",
            &mounts,
            "expected"
        )
        .unwrap()
    );
    assert!(
        !container_is_valid(
            &sample_inspect(Some("wrong"), "network"),
            "network",
            &mounts,
            "expected"
        )
        .unwrap()
    );
    assert!(
        !container_is_valid(
            &sample_inspect(Some("expected"), "old"),
            "network",
            &mounts,
            "expected"
        )
        .unwrap()
    );
}
