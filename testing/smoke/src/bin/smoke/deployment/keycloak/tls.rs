use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::Cursor,
    net::{Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow, ensure};
use rcgen::PublicKeyData;
use sha2::{Digest, Sha256};
use x509_parser::{extensions::GeneralName, pem::Pem};

use super::KEYCLOAK_CONTAINER_NAME;

const REQUIRED_SANS: &[&str] = &["localhost", "127.0.0.1", KEYCLOAK_CONTAINER_NAME];
const DIGEST_LENGTH: usize = 64;

#[derive(Debug, Clone)]
pub(super) struct TlsGeneration {
    pub(super) digest: String,
    pub(super) ca_path: PathBuf,
    pub(super) cert_path: PathBuf,
    pub(super) key_path: PathBuf,
}

/// Serializes every operation that observes or mutates one local Keycloak
/// instance. The lock lives in the host temporary directory and is keyed by
/// the fixed container identity, so profiles with different cluster names and
/// independent Git worktrees coordinate access to the shared container and port.
pub(super) struct StateLock {
    file: File,
}

impl StateLock {
    pub(super) fn acquire() -> Result<Self> {
        Self::acquire_at(&lock_path())
    }

    pub(super) fn acquire_at(path: &Path) -> Result<Self> {
        let parent = path
            .parent()
            .context("local Keycloak lock path has no parent directory")?;
        fs::create_dir_all(parent)
            .with_context(|| format!("creating Keycloak lock directory {}", parent.display()))?;
        restrict_lock_directory(parent)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .with_context(|| format!("opening local Keycloak lock {}", path.display()))?;
        file.lock()
            .with_context(|| format!("locking local Keycloak state {}", path.display()))?;
        Ok(Self { file })
    }
}

impl Drop for StateLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

fn lock_path() -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(b"veoveo.io/local-keycloak-lock/v1\0");
    hasher.update(KEYCLOAK_CONTAINER_NAME.as_bytes());
    let digest = hex::encode(hasher.finalize());
    std::env::temp_dir()
        .join("veoveo-local-keycloak-locks")
        .join(format!("{digest}.lock"))
}

fn restrict_lock_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .with_context(|| format!("restricting permissions on {}", path.display()))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

pub(super) fn state_dir(repository: &Path) -> PathBuf {
    repository.join("target").join("local-keycloak")
}

fn generations_dir(repository: &Path) -> PathBuf {
    state_dir(repository).join("generations")
}

fn current_pointer_path(repository: &Path) -> PathBuf {
    state_dir(repository).join("current")
}

fn generation_dir(repository: &Path, digest: &str) -> PathBuf {
    generations_dir(repository).join(digest)
}

fn generation_file_paths(generation_dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    (
        generation_dir.join("ca.pem"),
        generation_dir.join("tls.crt"),
        generation_dir.join("tls.key"),
    )
}

pub(super) fn active_generation(repository: &Path) -> Result<Option<TlsGeneration>> {
    let pointer_path = current_pointer_path(repository);
    let Ok(digest) = fs::read_to_string(&pointer_path) else {
        return Ok(None);
    };
    let digest = digest.trim();
    if !is_canonical_digest(digest) {
        return Ok(None);
    }

    let directory = generation_dir(repository, digest);
    let (ca_path, cert_path, key_path) = generation_file_paths(&directory);
    if validate_generation(&ca_path, &cert_path, &key_path).is_err() {
        return Ok(None);
    }
    let observed = generation_digest(
        &fs::read(&ca_path).with_context(|| format!("reading {}", ca_path.display()))?,
        &fs::read(&cert_path).with_context(|| format!("reading {}", cert_path.display()))?,
        &fs::read(&key_path).with_context(|| format!("reading {}", key_path.display()))?,
    );
    if observed != digest {
        return Ok(None);
    }

    Ok(Some(TlsGeneration {
        digest: digest.to_owned(),
        ca_path,
        cert_path,
        key_path,
    }))
}

fn is_canonical_digest(value: &str) -> bool {
    value.len() == DIGEST_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn ensure_generation(repository: &Path, _lock: &StateLock) -> Result<TlsGeneration> {
    cleanup_abandoned_temp_state(repository)?;
    if let Some(active) = active_generation(repository)? {
        return Ok(active);
    }
    generate_generation(repository)
}

fn generate_generation(repository: &Path) -> Result<TlsGeneration> {
    let generations = generations_dir(repository);
    fs::create_dir_all(&generations)
        .with_context(|| format!("creating {}", generations.display()))?;
    let temp_dir = generations.join(format!(".tmp-{}", unique_suffix()));
    fs::create_dir_all(&temp_dir).with_context(|| format!("creating {}", temp_dir.display()))?;

    let outcome = (|| -> Result<TlsGeneration> {
        let (ca_path, cert_path, key_path) = generation_file_paths(&temp_dir);
        generate_material(&ca_path, &cert_path, &key_path)?;
        restrict_key_permissions(&key_path)?;
        validate_generation(&ca_path, &cert_path, &key_path)
            .context("validating freshly generated local Keycloak TLS material")?;

        let digest = generation_digest(
            &fs::read(&ca_path).with_context(|| format!("reading {}", ca_path.display()))?,
            &fs::read(&cert_path).with_context(|| format!("reading {}", cert_path.display()))?,
            &fs::read(&key_path).with_context(|| format!("reading {}", key_path.display()))?,
        );
        let published_dir = generation_dir(repository, &digest);
        if published_dir.is_dir() {
            fs::remove_dir_all(&temp_dir).ok();
        } else {
            fs::rename(&temp_dir, &published_dir).with_context(|| {
                format!(
                    "publishing local Keycloak TLS generation {} to {}",
                    temp_dir.display(),
                    published_dir.display()
                )
            })?;
        }
        publish_current_pointer(repository, &digest)?;
        let (ca_path, cert_path, key_path) = generation_file_paths(&published_dir);
        Ok(TlsGeneration {
            digest,
            ca_path,
            cert_path,
            key_path,
        })
    })();

    if outcome.is_err() {
        let _ = fs::remove_dir_all(&temp_dir);
    }
    outcome
}

fn publish_current_pointer(repository: &Path, digest: &str) -> Result<()> {
    ensure!(
        is_canonical_digest(digest),
        "refusing to publish invalid TLS generation digest {digest:?}"
    );
    let state = state_dir(repository);
    fs::create_dir_all(&state).with_context(|| format!("creating {}", state.display()))?;
    let pointer_path = current_pointer_path(repository);
    let temp_path = state.join(format!(".current.tmp-{}", unique_suffix()));
    fs::write(&temp_path, digest).with_context(|| format!("writing {}", temp_path.display()))?;
    fs::rename(&temp_path, &pointer_path).with_context(|| {
        format!(
            "publishing local Keycloak current generation pointer {}",
            pointer_path.display()
        )
    })?;
    Ok(())
}

fn cleanup_abandoned_temp_state(repository: &Path) -> Result<()> {
    if let Ok(entries) = fs::read_dir(generations_dir(repository)) {
        for entry in entries.flatten() {
            if entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with(".tmp-"))
            {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
    }
    if let Ok(entries) = fs::read_dir(state_dir(repository)) {
        for entry in entries.flatten() {
            if entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with(".current.tmp-"))
            {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    Ok(())
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{nanos}", std::process::id())
}

fn generation_digest(ca_bytes: &[u8], cert_bytes: &[u8], key_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"veoveo.io/local-keycloak-generation/v1\0");
    hasher.update(ca_bytes);
    hasher.update([0]);
    hasher.update(cert_bytes);
    hasher.update([0]);
    hasher.update(key_bytes);
    hex::encode(hasher.finalize())
}

pub(super) fn generate_material(ca_path: &Path, cert_path: &Path, key_path: &Path) -> Result<()> {
    let mut ca_params = rcgen::CertificateParams::default();
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "Veoveo Local Keycloak CA");
    let ca_key = rcgen::KeyPair::generate()?;
    let ca_cert = ca_params.self_signed(&ca_key)?;

    let mut server_params = rcgen::CertificateParams::new(
        REQUIRED_SANS
            .iter()
            .map(|san| (*san).to_owned())
            .collect::<Vec<_>>(),
    )?;
    server_params.is_ca = rcgen::IsCa::ExplicitNoCa;
    server_params
        .extended_key_usages
        .push(rcgen::ExtendedKeyUsagePurpose::ServerAuth);
    let server_key = rcgen::KeyPair::generate()?;
    let ca_issuer = rcgen::Issuer::from_params(&ca_params, &ca_key);
    let server_cert = server_params.signed_by(&server_key, &ca_issuer)?;

    fs::write(ca_path, ca_cert.pem()).with_context(|| {
        format!(
            "writing local Keycloak CA certificate {}",
            ca_path.display()
        )
    })?;
    fs::write(cert_path, server_cert.pem()).with_context(|| {
        format!(
            "writing local Keycloak server certificate {}",
            cert_path.display()
        )
    })?;
    fs::write(key_path, server_key.serialize_pem())
        .with_context(|| format!("writing local Keycloak server key {}", key_path.display()))?;
    Ok(())
}

pub(super) fn restrict_key_permissions(key_path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(key_path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("restricting permissions on {}", key_path.display()))?;
    }
    #[cfg(not(unix))]
    {
        let _ = key_path;
    }
    Ok(())
}

pub(super) fn validate_generation(ca_path: &Path, cert_path: &Path, key_path: &Path) -> Result<()> {
    let ca_bytes = fs::read(ca_path).with_context(|| format!("reading {}", ca_path.display()))?;
    ensure!(!ca_bytes.is_empty(), "{} is empty", ca_path.display());
    let ca_pem = Pem::read(Cursor::new(ca_bytes))
        .map_err(|error| anyhow!("parsing CA PEM {}: {error}", ca_path.display()))?
        .0;
    let ca_cert = ca_pem
        .parse_x509()
        .map_err(|error| anyhow!("parsing CA certificate {}: {error}", ca_path.display()))?;
    ensure!(
        ca_cert.is_ca(),
        "local Keycloak CA {} does not set basicConstraints CA:true",
        ca_path.display()
    );

    let cert_bytes =
        fs::read(cert_path).with_context(|| format!("reading {}", cert_path.display()))?;
    ensure!(!cert_bytes.is_empty(), "{} is empty", cert_path.display());
    let cert_pem = Pem::read(Cursor::new(cert_bytes))
        .map_err(|error| {
            anyhow!(
                "parsing server certificate PEM {}: {error}",
                cert_path.display()
            )
        })?
        .0;
    let server_cert = cert_pem.parse_x509().map_err(|error| {
        anyhow!(
            "parsing server certificate {}: {error}",
            cert_path.display()
        )
    })?;
    ensure!(
        !server_cert.is_ca(),
        "local Keycloak server certificate {} must not be a CA",
        cert_path.display()
    );

    let observed_sans = server_cert
        .subject_alternative_name()
        .with_context(|| format!("reading SAN extension of {}", cert_path.display()))?
        .with_context(|| {
            format!(
                "server certificate {} omitted a Subject Alternative Name extension",
                cert_path.display()
            )
        })?
        .value
        .general_names
        .iter()
        .filter_map(general_name_value)
        .collect::<BTreeSet<_>>();
    for required in REQUIRED_SANS {
        ensure!(
            observed_sans.contains(*required),
            "local Keycloak server certificate {} is missing required SAN {required}; found {observed_sans:?}",
            cert_path.display()
        );
    }

    let extended_key_usage = server_cert
        .extended_key_usage()
        .with_context(|| format!("reading extended key usage of {}", cert_path.display()))?
        .with_context(|| {
            format!(
                "server certificate {} omitted an Extended Key Usage extension",
                cert_path.display()
            )
        })?;
    ensure!(
        extended_key_usage.value.server_auth,
        "local Keycloak server certificate {} omits the serverAuth extended key usage",
        cert_path.display()
    );
    server_cert
        .verify_signature(Some(ca_cert.public_key()))
        .with_context(|| {
            format!(
                "server certificate {} was not signed by CA {}",
                cert_path.display(),
                ca_path.display()
            )
        })?;

    validate_key_permissions(key_path)?;
    let key_text =
        fs::read_to_string(key_path).with_context(|| format!("reading {}", key_path.display()))?;
    ensure!(
        !key_text.trim().is_empty(),
        "{} is empty",
        key_path.display()
    );
    let key_pair = rcgen::KeyPair::from_pem(&key_text)
        .with_context(|| format!("parsing server private key {}", key_path.display()))?;
    ensure!(
        key_pair.subject_public_key_info() == server_cert.public_key().raw,
        "local Keycloak server private key {} does not match the certificate's public key {}",
        key_path.display(),
        cert_path.display()
    );
    Ok(())
}

fn general_name_value(name: &GeneralName<'_>) -> Option<String> {
    match name {
        GeneralName::DNSName(value) => Some((*value).to_owned()),
        GeneralName::IPAddress(bytes) => match bytes.len() {
            4 => Some(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]).to_string()),
            16 => <[u8; 16]>::try_from(*bytes)
                .ok()
                .map(Ipv6Addr::from)
                .map(|address| address.to_string()),
            _ => None,
        },
        _ => None,
    }
}

fn validate_key_permissions(key_path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(key_path)
            .with_context(|| format!("reading permissions of {}", key_path.display()))?
            .permissions()
            .mode();
        ensure!(
            mode & 0o077 == 0,
            "local Keycloak private key {} is accessible by group or other (mode {:o}); expected 0600",
            key_path.display(),
            mode & 0o777
        );
    }
    #[cfg(not(unix))]
    {
        let _ = key_path;
    }
    Ok(())
}

pub(super) fn remove_material(repository: &Path, _lock: &StateLock) -> Result<()> {
    let state = state_dir(repository);
    if state.is_dir() {
        fs::remove_dir_all(&state).with_context(|| {
            format!(
                "removing generated local Keycloak material {}",
                state.display()
            )
        })?;
    }
    Ok(())
}

#[cfg(test)]
pub(super) mod test_support {
    use super::*;

    pub(crate) fn current_pointer(repository: &Path) -> PathBuf {
        current_pointer_path(repository)
    }

    pub(crate) fn generation_directory(repository: &Path, digest: &str) -> PathBuf {
        generation_dir(repository, digest)
    }
}
