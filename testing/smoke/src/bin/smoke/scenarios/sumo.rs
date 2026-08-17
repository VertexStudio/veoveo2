use std::net::TcpListener;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use bytes::BytesMut;
use futures::StreamExt as _;
use re_grpc_server::{MemoryLimit, ServerOptions, shutdown};
use re_log_encoding::Decoder;
use re_log_types::{EntryId, LogMsg};
use re_protos::cloud::v1alpha1::{
    FindEntriesRequest, FindEntriesResponse, GetRrdManifestRequest, GetRrdManifestResponse,
    GetSegmentTableSchemaRequest, GetSegmentTableSchemaResponse, ReadDatasetEntryRequest,
    ReadDatasetEntryResponse, WhoAmIRequest, WhoAmIResponse,
};
use serde::Deserialize;
use veoveo_recording_hub::{
    DatasetName, DatasetRoute, SegmentReadScope, Spooler, SpoolerConfig, query_tree, run_blocking,
};
use veoveo_sumo_mcp::{
    driver::{FakeSimDriver, SimDriver},
    recording::RecordingPublisher,
};

use super::*;

const PLAYBACK_MANIFEST_SCHEMA: &str = "veoveo.io/recording-playback/v8";
const LIVE_RRD_CONTENT_TYPE: &str =
    "application/vnd.veoveo.rerun.rrd-stream; framing=be32; version=2";
const LIVE_RRD_START_HEADER: &str = "x-veoveo-rerun-live-start";
const MAX_LIVE_RRD_FRAME_BYTES: usize = 64 * 1024 * 1024;
const REDAP_TIMEOUT: Duration = Duration::from_secs(30);
const HISTORY_ARCHIVE_TIMEOUT: Duration = Duration::from_secs(90);
const HISTORY_ARCHIVE_POLL_INTERVAL: Duration = Duration::from_secs(2);
const MAX_RRD_MANIFEST_MESSAGES: usize = 256;
const MAX_REDAP_RESPONSE_BYTES: usize = 64 * 1024 * 1024;
const GRPC_WEB_CONTENT_TYPE: &str = "application/grpc-web+proto";

#[derive(Debug, Deserialize)]
struct SumoPlaybackManifest {
    schema: String,
    recording_id: String,
    state: String,
    access: SumoPlaybackAccess,
    archive: Option<SumoPlaybackArchive>,
    live: Option<SumoPlaybackLive>,
}

#[derive(Debug, Deserialize)]
struct SumoPlaybackAccess {
    redap_token: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SumoPlaybackArchive {
    uri: String,
    dataset_id: String,
    segment_id: String,
    byte_len: u64,
    layer_count: usize,
}

#[derive(Debug, Deserialize)]
struct SumoPlaybackLive {
    current_byte_len: u64,
    transport: String,
}

pub(crate) async fn sumo_push(steps: u32) -> Result<()> {
    ensure!(steps > 0, "steps must be positive");
    let temp = tempfile::tempdir()?;
    let spool_dir = temp.path().join("spool");
    let port = TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
    let bind = format!("127.0.0.1:{port}").parse()?;
    let config = SpoolerConfig {
        bind,
        spool_dir: spool_dir.clone(),
        datasets: vec![DatasetRoute {
            dataset: DatasetName::new("world")?,
            application_id_prefix: "veoveo-sumo".to_owned(),
        }],
        segment_max_bytes: 192 * 1024 * 1024,
        segment_max_age_s: 3_600,
        recording_idle_timeout_s: 15,
        flush_interval_ms: 10,
        fsync_on_flush: true,
        live_queue_limit_bytes: 256 * 1024 * 1024,
        blueprint_max_bytes: veoveo_recording_protocol::DEFAULT_MAXIMUM_BLUEPRINT_BYTES,
        blueprint_max_messages: veoveo_recording_protocol::DEFAULT_MAXIMUM_BLUEPRINT_MESSAGES,
        blueprint_max_revisions: veoveo_recording_protocol::DEFAULT_MAXIMUM_BLUEPRINT_REVISIONS,
    };
    let flush_interval = config.flush_interval();
    let max_age = config.segment_max_age();
    let (shutdown_signal, shutdown_handle) = shutdown::shutdown();
    let options = ServerOptions {
        memory_limit: MemoryLimit::from_bytes(config.live_queue_limit_bytes),
        ..Default::default()
    };
    let (receiver, _server) = re_grpc_server::spawn_with_recv(bind, options, shutdown_handle);
    let stopping = Arc::new(AtomicBool::new(false));
    let drain_stopping = stopping.clone();
    let drain = tokio::task::spawn_blocking(move || {
        run_blocking(
            Spooler::new(config)?,
            receiver,
            drain_stopping,
            flush_interval,
            max_age,
        )
    });

    tokio::time::sleep(Duration::from_millis(150)).await;
    let proxy = format!("rerun+http://127.0.0.1:{port}/proxy");
    let mut publisher = RecordingPublisher::connect(proxy, "sumo-smoke")?;
    let mut driver = FakeSimDriver::new(6, 3, (10, 20));
    publisher.publish_network(&driver.network_geometry()?)?;
    for _ in 0..steps {
        publisher.publish(&driver.state()?)?;
        driver.step(1)?;
    }
    publisher.flush()?;
    drop(publisher);
    tokio::time::sleep(Duration::from_millis(400)).await;

    stopping.store(true, Ordering::SeqCst);
    shutdown_signal.stop();
    drain.await.context("SUMO recording drain panicked")??;

    let query = query_tree(
        &spool_dir.join("world"),
        "/world/sumo/**",
        "tick",
        u64::from(steps) + 1,
        SegmentReadScope::Frozen,
    )?;
    ensure!(
        query.rows_by_recording.get("sumo-smoke") == Some(&u64::from(steps)),
        "expected {steps} durable SUMO rows, got {:?}",
        query.rows_by_recording
    );
    println!("sumo push smoke ok: {steps} typed world frames persisted and queried");
    Ok(())
}

pub(crate) async fn sumo_verify(conformance: &Path, context: &str) -> Result<()> {
    assert_executable(conformance)?;

    run_checked(
        Path::new("kubectl"),
        ["--context".into(), context.into(), "cluster-info".into()],
        [],
    )
    .context("SUMO verification requires the active k3d cluster")?;

    let mcp_url = "http://127.0.0.1:8895/sumo/mcp";
    let health_url = "http://127.0.0.1:8895/sumo/healthz";
    let client = reqwest::Client::new();
    let mut ready = false;
    for _ in 0..300 {
        if client
            .get(health_url)
            .send()
            .await
            .is_ok_and(|response| response.status() == StatusCode::OK)
        {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    if !ready {
        let logs = run_checked(
            Path::new("kubectl"),
            [
                "--context".into(),
                context.into(),
                "-n".into(),
                "veoveo".into(),
                "logs".into(),
                "deployment/sumo-mcp".into(),
                "--tail=200".into(),
            ],
            [],
        )
        .unwrap_or_else(|error| error.to_string());
        bail!("SUMO MCP did not become healthy\n{logs}");
    }

    assert_http_get_status(mcp_url, None, StatusCode::UNAUTHORIZED).await?;
    let auth = [(
        "VEOVEO_INTERNAL_SIGNING_KEY_DER_B64",
        INTERNAL_SIGNING_KEY_DER_B64.into(),
    )];
    let base = [
        "--url",
        mcp_url,
        "--scheme",
        "sumo",
        "--internal-server",
        "sumo",
    ];
    let info = run_conformance(conformance, &base, &["info"], auth.clone())?;
    contains(&info, "run_batch")?;
    let resources = run_conformance(conformance, &base, &["resources"], auth.clone())?;
    contains(&resources, "sumo://congestion")?;

    let state = run_conformance(
        conformance,
        &base,
        &["call", "--tool-name", "query_state", "--arguments", "{}"],
        auth.clone(),
    )?;
    let state = structured_output(&state)?;
    ensure!(
        state.get("vehicle_count").and_then(Value::as_u64).is_some(),
        "query_state did not return a typed vehicle_count: {state}"
    );

    let scenario = run_conformance(
        conformance,
        &base,
        &[
            "call",
            "--tool-name",
            "describe_scenario",
            "--arguments",
            "{}",
        ],
        auth.clone(),
    )?;
    let scenario = structured_output(&scenario)?;
    let edge = scenario
        .get("edges")
        .and_then(Value::as_array)
        .and_then(|edges| edges.first())
        .and_then(Value::as_str)
        .context("live SUMO scenario exposed no edges")?;
    let edge_request = serde_json::json!({"edge_id": edge, "speed_mps": 8.0}).to_string();
    let actuation = run_conformance(
        conformance,
        &base,
        &[
            "call",
            "--tool-name",
            "set_edge_speed",
            "--arguments",
            &edge_request,
        ],
        auth.clone(),
    )?;
    ensure!(
        structured_output(&actuation)?
            .get("applied")
            .and_then(Value::as_bool)
            == Some(true),
        "live SUMO actuation was not applied"
    );

    let task = run_conformance(
        conformance,
        &base,
        &[
            "task-call",
            "--tool-name",
            "run_batch",
            "--arguments",
            r#"{"steps":50}"#,
        ],
        auth,
    )?;
    let task_result = structured_output(&task)?;
    ensure!(
        task_result.get("steps_advanced").and_then(Value::as_u64) == Some(50),
        "run_batch task did not advance 50 steps: {task_result}"
    );

    tokio::time::sleep(Duration::from_secs(2)).await;
    let pod = run_checked(
        Path::new("kubectl"),
        [
            "--context".into(),
            context.into(),
            "-n".into(),
            "veoveo".into(),
            "get".into(),
            "pod".into(),
            "-l".into(),
            "app.kubernetes.io/component=recording".into(),
            "-o".into(),
            "jsonpath={.items[0].metadata.name}".into(),
        ],
        [],
    )?;
    let query = run_checked(
        Path::new("kubectl"),
        [
            "--context".into(),
            context.into(),
            "-n".into(),
            "veoveo".into(),
            "exec".into(),
            pod.trim().into(),
            "-c".into(),
            "recording-hub".into(),
            "--".into(),
            "hub-query".into(),
            "--root".into(),
            "/recordings".into(),
            "--include-active".into(),
            "--entities".into(),
            "/world/sumo/**".into(),
            "--timeline".into(),
            "tick".into(),
            "--max-rows".into(),
            "0".into(),
        ],
        [],
    )?;
    let query: Value = serde_json::from_str(query.trim())?;
    let rows = query
        .get("rows_by_recording")
        .and_then(Value::as_object)
        .context("hub query omitted rows_by_recording")?;
    ensure!(
        rows.iter().any(|(recording, count)| {
            recording.starts_with("sumo-live") && count.as_u64().is_some_and(|count| count > 0)
        }),
        "Recording Hub did not retain the live SUMO world: {rows:?}"
    );

    verify_console_keycloak_login("http://localhost:8780").await?;

    println!(
        "sumo verify ok: live k3d TraCI, authenticated MCP task/actuation, durable world, and Console Keycloak login"
    );
    Ok(())
}

async fn verify_console_keycloak_login(console_base_url: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(Policy::none())
        .timeout(Duration::from_secs(15))
        .build()?;

    // 1. Hit Console BFF /auth/login
    let response = client
        .get(format!("{console_base_url}/auth/login"))
        .send()
        .await
        .context("connecting to Console BFF /auth/login")?;
    let status = response.status();
    ensure!(
        status == StatusCode::SEE_OTHER || status == StatusCode::FOUND,
        "Console /auth/login returned {status}, expected 303 or 302"
    );
    let authorize_location = response
        .headers()
        .get(LOCATION)
        .and_then(|val| val.to_str().ok())
        .context("Console /auth/login omitted Location header")?
        .to_owned();

    // 2. Follow redirect to Gateway /oauth/authorize
    let response = client
        .get(&authorize_location)
        .send()
        .await
        .context("connecting to Gateway /oauth/authorize")?;
    let status = response.status();
    ensure!(
        status == StatusCode::FOUND || status == StatusCode::SEE_OTHER,
        "Gateway /oauth/authorize returned {status}, expected 302"
    );
    let idp_auth_location = response
        .headers()
        .get(LOCATION)
        .and_then(|val| val.to_str().ok())
        .context("Gateway /oauth/authorize omitted Location header")?
        .to_owned();

    // 3. Load Keycloak login page
    let response = client
        .get(&idp_auth_location)
        .send()
        .await
        .context("connecting to Keycloak authorization endpoint")?;
    ensure!(
        response.status() == StatusCode::OK,
        "Keycloak authorization page returned {}",
        response.status()
    );
    let html = response.text().await?;
    let document = scraper::Html::parse_document(&html);
    let selector = scraper::Selector::parse("form#kc-form-login")
        .map_err(|err| anyhow!("invalid Keycloak login form selector: {err}"))?;
    let form_action = document
        .select(&selector)
        .next()
        .and_then(|form| form.value().attr("action"))
        .context("Keycloak login form omitted action attribute")?;
    let parsed_action = reqwest::Url::parse(form_action)
        .or_else(|_| reqwest::Url::parse(&idp_auth_location)?.join(form_action))?;

    // 4. Submit login credentials (alice / keycloak-local-password)
    let form_body = form_urlencoded(&[
        ("username", "alice"),
        ("password", "keycloak-local-password"),
        ("credentialId", ""),
    ]);
    let response = client
        .post(parsed_action)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(form_body)
        .send()
        .await
        .context("submitting Keycloak login credentials")?;
    let status = response.status();
    ensure!(
        status == StatusCode::FOUND || status == StatusCode::SEE_OTHER,
        "Keycloak login POST returned {status}, expected redirect"
    );
    let gateway_callback_url = response
        .headers()
        .get(LOCATION)
        .and_then(|val| val.to_str().ok())
        .context("Keycloak login omitted Location header")?
        .to_owned();

    // 5. Follow redirect to Gateway /oauth/callback
    let response = client
        .get(&gateway_callback_url)
        .send()
        .await
        .context("connecting to Gateway /oauth/callback")?;
    let status = response.status();
    ensure!(
        status == StatusCode::FOUND || status == StatusCode::SEE_OTHER,
        "Gateway /oauth/callback returned {status}, expected redirect to Console /auth/callback"
    );
    let console_callback_url = response
        .headers()
        .get(LOCATION)
        .and_then(|val| val.to_str().ok())
        .context("Gateway /oauth/callback omitted Location header")?
        .to_owned();

    // 6. Follow redirect to Console BFF /auth/callback
    let response = client
        .get(&console_callback_url)
        .send()
        .await
        .context("connecting to Console /auth/callback")?;
    let status = response.status();
    ensure!(
        status == StatusCode::SEE_OTHER || status == StatusCode::FOUND,
        "Console /auth/callback returned {status}, expected redirect to /console/"
    );

    // 7. Verify Console /console/ loads with the authenticated session
    let response = client
        .get(format!("{console_base_url}/console/"))
        .send()
        .await
        .context("accessing /console/ with authenticated session")?;
    ensure!(
        response.status() == StatusCode::OK,
        "Console /console/ returned {}, expected 200 OK",
        response.status()
    );

    let response = client
        .get(format!("{console_base_url}/console/api/snapshot"))
        .send()
        .await
        .context("loading authenticated Console snapshot")?;
    ensure!(
        response.status() == StatusCode::OK,
        "Console snapshot returned {}, expected 200 OK",
        response.status()
    );
    let snapshot: Value = response.json().await.context("decoding Console snapshot")?;
    let recording_id = snapshot
        .get("recordings")
        .and_then(Value::as_array)
        .and_then(|recordings| {
            recordings.iter().find_map(|recording| {
                recording
                    .get("recordingKey")
                    .and_then(Value::as_str)
                    .filter(|key| key.starts_with("sumo-live"))
                    .and_then(|_| recording.get("id"))
                    .and_then(Value::as_str)
            })
        })
        .context("Console snapshot omitted the live SUMO recording")?;

    let playback_url = format!("{console_base_url}/console/api/recordings/{recording_id}/playback");
    let manifest = fetch_playback_manifest(&client, &playback_url, recording_id).await?;
    verify_live_rrd_stream(&client, console_base_url, recording_id, &manifest.live).await?;

    let (archive, redap_token) =
        wait_for_history_archive(&client, &playback_url, recording_id, manifest).await?;
    ensure!(
        archive.uri.starts_with("rerun+http"),
        "Console SUMO History archive used an invalid Redap URI: {}",
        archive.uri
    );
    ensure!(
        !archive.dataset_id.is_empty() && archive.byte_len > 0 && archive.layer_count > 0,
        "Console SUMO History archive is empty or incomplete: {archive:?}"
    );
    verify_history_redap(&archive, &redap_token).await?;
    println!(
        "Console login, SUMO History archive, and Live RRD stream verified ok via local Keycloak at {console_base_url}"
    );
    Ok(())
}

async fn fetch_playback_manifest(
    client: &reqwest::Client,
    playback_url: &str,
    recording_id: &str,
) -> Result<SumoPlaybackManifest> {
    let response = client
        .get(playback_url)
        .send()
        .await
        .context("loading live SUMO playback manifest")?;
    ensure!(
        response.status() == StatusCode::OK,
        "Console SUMO playback manifest returned {}, expected 200 OK",
        response.status()
    );
    let manifest: SumoPlaybackManifest = response
        .json()
        .await
        .context("decoding live SUMO playback manifest")?;
    validate_playback_manifest(&manifest, recording_id)?;
    Ok(manifest)
}

fn validate_playback_manifest(manifest: &SumoPlaybackManifest, recording_id: &str) -> Result<()> {
    ensure!(
        manifest.schema == PLAYBACK_MANIFEST_SCHEMA,
        "Console SUMO playback manifest used unsupported schema {}",
        manifest.schema
    );
    ensure!(
        manifest.recording_id == recording_id,
        "Console SUMO playback manifest targeted another recording: {}",
        manifest.recording_id
    );
    ensure!(
        manifest.state == "live",
        "Console SUMO recording is not live: {}",
        manifest.state
    );
    let live = manifest
        .live
        .as_ref()
        .context("Console SUMO playback manifest omitted Live segment")?;
    ensure!(
        live.transport == "rerun_rrd_channel_v2" && live.current_byte_len > 0,
        "Console SUMO Live segment is empty or unsupported: {live:?}"
    );
    Ok(())
}

async fn wait_for_history_archive(
    client: &reqwest::Client,
    playback_url: &str,
    recording_id: &str,
    mut manifest: SumoPlaybackManifest,
) -> Result<(SumoPlaybackArchive, String)> {
    let started = Instant::now();
    let mut last_summary =
        match evaluate_history_manifest(&manifest, recording_id, started.elapsed())? {
            HistoryArchiveDecision::Pending { summary } => summary,
            HistoryArchiveDecision::Ready {
                archive,
                redap_token,
            } => return Ok((archive.clone(), redap_token.to_owned())),
        };
    loop {
        let remaining = HISTORY_ARCHIVE_TIMEOUT.saturating_sub(started.elapsed());
        ensure!(
            !remaining.is_zero(),
            "Console SUMO History archive did not appear for recording {recording_id} within {:?}; last manifest: {last_summary}",
            HISTORY_ARCHIVE_TIMEOUT
        );
        let next_manifest = tokio::time::timeout(remaining, async {
            tokio::time::sleep(HISTORY_ARCHIVE_POLL_INTERVAL.min(remaining)).await;
            fetch_playback_manifest(client, playback_url, recording_id).await
        })
        .await
        .map_err(|_| anyhow::anyhow!(
            "Console SUMO History archive did not appear for recording {recording_id} within {:?}; last manifest: {last_summary}",
            HISTORY_ARCHIVE_TIMEOUT
        ))??;
        manifest = next_manifest;
        match evaluate_history_manifest(&manifest, recording_id, started.elapsed())? {
            HistoryArchiveDecision::Ready {
                archive,
                redap_token,
            } => return Ok((archive.clone(), redap_token.to_owned())),
            HistoryArchiveDecision::Pending { summary } => last_summary = summary,
        }
    }
}

#[derive(Debug)]
enum HistoryArchiveDecision<'a> {
    Pending {
        summary: String,
    },
    Ready {
        archive: &'a SumoPlaybackArchive,
        redap_token: &'a str,
    },
}

fn evaluate_history_manifest<'a>(
    manifest: &'a SumoPlaybackManifest,
    recording_id: &str,
    elapsed: Duration,
) -> Result<HistoryArchiveDecision<'a>> {
    validate_playback_manifest(manifest, recording_id)?;
    if elapsed >= HISTORY_ARCHIVE_TIMEOUT {
        let summary = playback_summary(manifest);
        anyhow::bail!(
            "Console SUMO History archive did not appear for recording {recording_id} within {:?}; last manifest: {summary}",
            HISTORY_ARCHIVE_TIMEOUT
        );
    }
    if let Some(archive) = manifest.archive.as_ref() {
        return Ok(HistoryArchiveDecision::Ready {
            archive,
            redap_token: &manifest.access.redap_token,
        });
    }
    let summary = playback_summary(manifest);
    Ok(HistoryArchiveDecision::Pending { summary })
}

fn playback_summary(manifest: &SumoPlaybackManifest) -> String {
    format!(
        "state={}, archive={}, live_bytes={}, live_transport={}",
        manifest.state,
        if manifest.archive.is_some() {
            "present"
        } else {
            "pending"
        },
        manifest
            .live
            .as_ref()
            .map_or(0, |live| live.current_byte_len),
        manifest
            .live
            .as_ref()
            .map_or("missing", |live| live.transport.as_str())
    )
}

async fn verify_live_rrd_stream(
    client: &reqwest::Client,
    console_base_url: &str,
    recording_id: &str,
    live: &Option<SumoPlaybackLive>,
) -> Result<()> {
    let live = live
        .as_ref()
        .context("Console SUMO playback manifest omitted Live segment")?;
    ensure!(
        live.transport == "rerun_rrd_channel_v2" && live.current_byte_len > 0,
        "Console SUMO Live segment is empty or unsupported: {live:?}"
    );
    let response = client
        .get(format!(
            "{console_base_url}/console/api/recordings/{recording_id}/live/rrd-stream"
        ))
        .header(reqwest::header::ACCEPT, LIVE_RRD_CONTENT_TYPE)
        .header(LIVE_RRD_START_HEADER, "bootstrap")
        .send()
        .await
        .context("opening Console SUMO Live RRD stream")?;
    ensure!(
        response.status() == StatusCode::OK,
        "Console SUMO Live RRD stream returned {}, expected 200 OK",
        response.status()
    );
    ensure!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            == Some(LIVE_RRD_CONTENT_TYPE),
        "Console SUMO Live RRD stream returned unexpected content type {:?}",
        response.headers().get(CONTENT_TYPE)
    );
    let messages = read_one_live_rrd_frame(response).await?;
    ensure!(
        !messages.is_empty(),
        "Console SUMO Live RRD stream produced an empty frame"
    );
    Ok(())
}

async fn verify_history_redap(archive: &SumoPlaybackArchive, token: &str) -> Result<()> {
    ensure!(
        !token.is_empty(),
        "Console SUMO playback manifest omitted its Redap token"
    );
    let dataset_id: EntryId = archive
        .dataset_id
        .parse()
        .context("Console SUMO History archive used an invalid dataset ID")?;
    let endpoint = redap_endpoint(&archive.uri, &archive.dataset_id, &archive.segment_id)?;
    let client = reqwest::Client::builder().timeout(REDAP_TIMEOUT).build()?;

    let identity: WhoAmIResponse =
        grpc_web_unary(&client, &endpoint, "WhoAmI", &WhoAmIRequest {}, token, None).await?;
    ensure!(
        identity.can_read && !identity.can_write,
        "Console SUMO History Redap granted unexpected access: read={}, write={}",
        identity.can_read,
        identity.can_write
    );

    let entries: FindEntriesResponse = grpc_web_unary(
        &client,
        &endpoint,
        "FindEntries",
        &FindEntriesRequest::default(),
        token,
        None,
    )
    .await?;
    let expected_proto_id = dataset_id.into();
    ensure!(
        entries.entries.len() == 1 && entries.entries[0].id.as_ref() == Some(&expected_proto_id),
        "Console SUMO History Redap catalog did not resolve exactly dataset {}",
        archive.dataset_id
    );

    let dataset: ReadDatasetEntryResponse = grpc_web_unary(
        &client,
        &endpoint,
        "ReadDatasetEntry",
        &ReadDatasetEntryRequest {},
        token,
        Some(dataset_id),
    )
    .await?;
    let dataset = dataset
        .dataset
        .context("Console SUMO History Redap omitted the dataset entry")?;
    ensure!(
        dataset
            .details
            .as_ref()
            .and_then(|details| details.id.as_ref())
            == Some(&expected_proto_id),
        "Console SUMO History Redap returned another dataset"
    );

    let schema: GetSegmentTableSchemaResponse = grpc_web_unary(
        &client,
        &endpoint,
        "GetSegmentTableSchema",
        &GetSegmentTableSchemaRequest {},
        token,
        Some(dataset_id),
    )
    .await?;
    let schema = schema
        .schema
        .and_then(|schema| schema.arrow_schema)
        .context("Console SUMO History Redap omitted the segment table schema")?;
    ensure!(
        !schema.is_empty(),
        "Console SUMO History Redap returned an empty segment table schema"
    );

    let manifests: Vec<GetRrdManifestResponse> = grpc_web_call(
        &client,
        &endpoint,
        "GetRrdManifest",
        &GetRrdManifestRequest {
            segment_id: Some(archive.segment_id.clone().into()),
        },
        token,
        Some(dataset_id),
    )
    .await?;
    ensure!(
        !manifests.is_empty() && manifests.len() <= MAX_RRD_MANIFEST_MESSAGES,
        "Console SUMO History Redap returned an invalid RRD manifest count: {}",
        manifests.len()
    );
    for response in manifests {
        let manifest = response
            .rrd_manifest
            .context("Console SUMO History Redap returned an empty RRD manifest message")?;
        ensure!(
            manifest.store_id.is_some()
                && manifest.sorbet_schema.is_some()
                && manifest.data.is_some(),
            "Console SUMO History Redap returned an incomplete RRD manifest"
        );
    }
    Ok(())
}

fn redap_endpoint(
    uri: &str,
    expected_dataset_id: &str,
    expected_segment_id: &str,
) -> Result<String> {
    let dataset_uri: re_uri::DatasetSegmentUri = uri
        .parse()
        .context("parsing Console SUMO History Redap URI")?;
    ensure!(
        dataset_uri.dataset_id.to_string() == expected_dataset_id,
        "Console SUMO History Redap URI targeted another dataset: {uri}"
    );
    ensure!(
        dataset_uri.segment_id.as_ref() == expected_segment_id,
        "Console SUMO History Redap URI targeted another segment: {uri}"
    );
    Ok(dataset_uri.origin.as_url())
}

async fn grpc_web_unary<Q, S>(
    client: &reqwest::Client,
    endpoint: &str,
    method: &str,
    request: &Q,
    token: &str,
    entry_id: Option<EntryId>,
) -> Result<S>
where
    Q: prost::Message,
    S: prost::Message + Default,
{
    let mut responses = grpc_web_call(client, endpoint, method, request, token, entry_id).await?;
    ensure!(
        responses.len() == 1,
        "Console SUMO History Redap {method} returned {} messages, expected one",
        responses.len()
    );
    Ok(responses.pop().expect("one response was checked"))
}

async fn grpc_web_call<Q, S>(
    client: &reqwest::Client,
    endpoint: &str,
    method: &str,
    request: &Q,
    token: &str,
    entry_id: Option<EntryId>,
) -> Result<Vec<S>>
where
    Q: prost::Message,
    S: prost::Message + Default,
{
    let encoded_len = request.encoded_len();
    let encoded_len_u32 =
        u32::try_from(encoded_len).context("Redap request exceeds gRPC framing")?;
    let mut body = Vec::with_capacity(5 + encoded_len);
    body.push(0);
    body.extend_from_slice(&encoded_len_u32.to_be_bytes());
    request
        .encode(&mut body)
        .context("encoding Console SUMO History Redap request")?;

    let mut builder = client
        .post(format!(
            "{endpoint}/rerun.cloud.v1alpha1.RerunCloudService/{method}"
        ))
        .header(reqwest::header::CONTENT_TYPE, GRPC_WEB_CONTENT_TYPE)
        .header(reqwest::header::ACCEPT, GRPC_WEB_CONTENT_TYPE)
        .bearer_auth(token)
        .body(body);
    if let Some(entry_id) = entry_id {
        builder = builder.header("x-rerun-entry-id", entry_id.to_string());
    }
    let response = builder
        .send()
        .await
        .with_context(|| format!("calling Console SUMO History Redap {method}"))?;
    ensure!(
        response.status() == StatusCode::OK,
        "Console SUMO History Redap {method} returned HTTP {}",
        response.status()
    );
    ensure!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("application/grpc-web")),
        "Console SUMO History Redap {method} returned unexpected content type {:?}",
        response.headers().get(CONTENT_TYPE)
    );

    let mut stream = response.bytes_stream();
    let mut framed = BytesMut::new();
    while let Some(chunk) = stream.next().await {
        framed.extend_from_slice(
            &chunk.with_context(|| format!("reading Console SUMO History Redap {method}"))?,
        );
        ensure!(
            framed.len() <= MAX_REDAP_RESPONSE_BYTES,
            "Console SUMO History Redap {method} exceeded {MAX_REDAP_RESPONSE_BYTES} bytes"
        );
    }
    decode_grpc_web_frames(&framed, method)
}

fn decode_grpc_web_frames<S>(framed: &[u8], method: &str) -> Result<Vec<S>>
where
    S: prost::Message + Default,
{
    let mut offset = 0_usize;
    let mut responses = Vec::new();
    let mut saw_success_trailers = false;
    while offset < framed.len() {
        ensure!(
            framed.len() - offset >= 5,
            "Console SUMO History Redap {method} ended inside a gRPC-Web frame header"
        );
        let flags = framed[offset];
        let frame_len = u32::from_be_bytes(
            framed[offset + 1..offset + 5]
                .try_into()
                .expect("four-byte gRPC-Web length"),
        ) as usize;
        offset += 5;
        ensure!(
            frame_len <= framed.len() - offset,
            "Console SUMO History Redap {method} ended inside a gRPC-Web frame"
        );
        let payload = &framed[offset..offset + frame_len];
        offset += frame_len;
        if flags == 0 {
            responses.push(
                S::decode(payload)
                    .with_context(|| format!("decoding Console SUMO History Redap {method}"))?,
            );
            ensure!(
                responses.len() <= MAX_RRD_MANIFEST_MESSAGES,
                "Console SUMO History Redap {method} exceeded the bounded message count"
            );
        } else if flags == 0x80 {
            let trailers = std::str::from_utf8(payload).with_context(|| {
                format!("decoding Console SUMO History Redap {method} trailers")
            })?;
            ensure!(
                trailers.lines().any(|line| {
                    line.split_once(':').is_some_and(|(name, value)| {
                        name.eq_ignore_ascii_case("grpc-status") && value.trim() == "0"
                    })
                }),
                "Console SUMO History Redap {method} failed: {}",
                trailers.trim()
            );
            saw_success_trailers = true;
        } else {
            bail!("Console SUMO History Redap {method} used unsupported frame flags {flags:#04x}");
        }
    }
    ensure!(
        saw_success_trailers || !responses.is_empty(),
        "Console SUMO History Redap {method} omitted both response data and success trailers"
    );
    Ok(responses)
}

async fn read_one_live_rrd_frame(response: reqwest::Response) -> Result<Vec<LogMsg>> {
    tokio::time::timeout(Duration::from_secs(30), async move {
        let mut body = response.bytes_stream();
        let mut framed = BytesMut::new();
        let frame_len = loop {
            if framed.len() >= 4 {
                let len = u32::from_be_bytes(framed[..4].try_into().expect("four-byte prefix"));
                let len = usize::try_from(len).context("RRD frame length does not fit usize")?;
                ensure!(
                    len > 0 && len <= MAX_LIVE_RRD_FRAME_BYTES,
                    "Console SUMO Live RRD frame length {len} is outside 1..={MAX_LIVE_RRD_FRAME_BYTES}"
                );
                break len;
            }
            let chunk = body
                .next()
                .await
                .context("Console SUMO Live RRD stream ended before its frame prefix")?
                .context("reading Console SUMO Live RRD frame prefix")?;
            framed.extend_from_slice(&chunk);
        };
        while framed.len() < 4 + frame_len {
            let chunk = body
                .next()
                .await
                .context("Console SUMO Live RRD stream ended inside a frame")?
                .context("reading Console SUMO Live RRD frame payload")?;
            framed.extend_from_slice(&chunk);
            ensure!(
                framed.len() <= 4 + MAX_LIVE_RRD_FRAME_BYTES,
                "Console SUMO Live RRD stream exceeded the bounded frame buffer"
            );
        }
        let frame = framed.split_to(4 + frame_len).freeze();
        Decoder::<LogMsg>::decode_eager(std::io::Cursor::new(&frame[4..]))
            .context("opening complete Console SUMO Live RRD frame")?
            .collect::<Result<Vec<_>, _>>()
            .context("decoding complete Console SUMO Live RRD frame")
    })
    .await
    .context("timed out waiting for one complete Console SUMO Live RRD frame")?
}

fn structured_output(output: &str) -> Result<Value> {
    let raw = output
        .lines()
        .find_map(|line| line.strip_prefix("structured: "))
        .context("conformance output omitted structured content")?;
    serde_json::from_str(raw).context("parsing conformance structured content")
}

fn run_conformance<const N: usize>(
    conformance: &Path,
    base: &[&str],
    command: &[&str],
    environment: [(&'static str, OsString); N],
) -> Result<String> {
    let arguments = base
        .iter()
        .chain(command)
        .map(OsString::from)
        .collect::<Vec<_>>();
    run_checked(conformance, arguments, environment)
}

#[cfg(test)]
mod redap_history_tests {
    use super::*;
    use prost::Message as _;

    const SEGMENT_ID: &str = "sumo-live";
    const RECORDING_ID: &str = "sumo-live-1";

    fn manifest(archive: Option<SumoPlaybackArchive>) -> SumoPlaybackManifest {
        SumoPlaybackManifest {
            schema: PLAYBACK_MANIFEST_SCHEMA.to_owned(),
            recording_id: RECORDING_ID.to_owned(),
            state: "live".to_owned(),
            access: SumoPlaybackAccess {
                redap_token: "token".to_owned(),
            },
            archive,
            live: Some(SumoPlaybackLive {
                current_byte_len: 1,
                transport: "rerun_rrd_channel_v2".to_owned(),
            }),
        }
    }

    fn archive() -> SumoPlaybackArchive {
        SumoPlaybackArchive {
            uri: "rerun+http://localhost:8780/dataset/id?segment_id=sumo-live".to_owned(),
            dataset_id: "id".to_owned(),
            segment_id: SEGMENT_ID.to_owned(),
            byte_len: 1,
            layer_count: 1,
        }
    }

    #[test]
    fn playback_state_allows_pending_history_then_accepts_archive() {
        let pending = manifest(None);
        let decision =
            evaluate_history_manifest(&pending, RECORDING_ID, Duration::from_secs(1)).unwrap();
        assert!(matches!(
            decision,
            HistoryArchiveDecision::Pending { summary } if summary.contains("archive=pending")
        ));

        let ready = manifest(Some(archive()));
        let decision =
            evaluate_history_manifest(&ready, RECORDING_ID, Duration::from_secs(1)).unwrap();
        match decision {
            HistoryArchiveDecision::Ready {
                archive,
                redap_token,
            } => {
                assert_eq!(archive.segment_id, SEGMENT_ID);
                assert_eq!(redap_token, "token");
            }
            HistoryArchiveDecision::Pending { .. } => panic!("archive remained pending"),
        }
    }

    #[test]
    fn pending_history_times_out_with_context_and_without_secret() {
        let pending = manifest(None);
        let error =
            evaluate_history_manifest(&pending, RECORDING_ID, HISTORY_ARCHIVE_TIMEOUT).unwrap_err();
        let message = error.to_string();
        assert!(message.contains(RECORDING_ID));
        assert!(message.contains("90s"));
        assert!(message.contains("archive=pending"));
        assert!(!message.contains("token"));
    }

    #[test]
    fn archive_published_at_deadline_is_not_accepted_late() {
        let ready = manifest(Some(archive()));
        let error =
            evaluate_history_manifest(&ready, RECORDING_ID, HISTORY_ARCHIVE_TIMEOUT).unwrap_err();
        let message = error.to_string();
        assert!(message.contains(RECORDING_ID));
        assert!(message.contains("archive=present"));
    }

    #[test]
    fn invalid_manifest_fails_before_pending_decision() {
        let mut invalid = manifest(None);
        invalid.schema = "wrong".to_owned();
        let error = evaluate_history_manifest(&invalid, RECORDING_ID, Duration::ZERO).unwrap_err();
        assert!(error.to_string().contains("unsupported schema"));

        let mut invalid_live = manifest(None);
        invalid_live.live = None;
        let error =
            evaluate_history_manifest(&invalid_live, RECORDING_ID, Duration::ZERO).unwrap_err();
        assert!(error.to_string().contains("omitted Live segment"));
    }

    #[test]
    fn redap_uri_requires_exact_dataset_and_segment() {
        let dataset_id = EntryId::new().to_string();
        let uri =
            format!("rerun+http://localhost:8780/dataset/{dataset_id}?segment_id={SEGMENT_ID}");
        assert_eq!(
            redap_endpoint(&uri, &dataset_id, SEGMENT_ID).unwrap(),
            "http://localhost:8780"
        );
        assert!(redap_endpoint(&uri, &EntryId::new().to_string(), SEGMENT_ID).is_err());
        assert!(redap_endpoint(&uri, &dataset_id, "another-segment").is_err());
    }

    #[test]
    fn grpc_web_decoder_accepts_data_and_compact_success_trailers() {
        let response = WhoAmIResponse {
            user_id: Some("alice".to_owned()),
            can_read: true,
            can_write: false,
        };
        let mut framed = grpc_web_frame(0, &response.encode_to_vec());
        framed.extend(grpc_web_frame(0x80, b"grpc-status:0\r\n"));
        let decoded: Vec<WhoAmIResponse> = decode_grpc_web_frames(&framed, "WhoAmI").unwrap();
        assert_eq!(decoded, vec![response]);
    }

    #[test]
    fn grpc_web_decoder_rejects_error_and_truncated_frames() {
        let failure = grpc_web_frame(0x80, b"grpc-status:7\r\ngrpc-message:denied\r\n");
        assert!(decode_grpc_web_frames::<WhoAmIResponse>(&failure, "WhoAmI").is_err());

        let mut truncated = grpc_web_frame(0, &WhoAmIResponse::default().encode_to_vec());
        truncated.pop();
        assert!(decode_grpc_web_frames::<WhoAmIResponse>(&truncated, "WhoAmI").is_err());
    }

    fn grpc_web_frame(flags: u8, payload: &[u8]) -> Vec<u8> {
        let mut frame = Vec::with_capacity(5 + payload.len());
        frame.push(flags);
        frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        frame.extend_from_slice(payload);
        frame
    }
}
