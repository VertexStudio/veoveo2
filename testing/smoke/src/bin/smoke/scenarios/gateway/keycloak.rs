use scraper::{Html, Selector};

use super::*;

const KEYCLOAK_IMAGE: &str = "quay.io/keycloak/keycloak@sha256:0aae0de7fca85525f727d3354df17896092de8bb26ae4c12d89c77e5df8cbce4";
const KEYCLOAK_REALM: &str = "veoveo-ci";
const KEYCLOAK_CLIENT_ID: &str = "veoveo-gateway";
const KEYCLOAK_CLIENT_SECRET: &str = "keycloak-ci-oidc-client-secret";
const KEYCLOAK_USERNAME: &str = "alice";
const KEYCLOAK_PASSWORD: &str = "keycloak-ci-password";

pub(crate) async fn gateway_keycloak(
    conformance: &Path,
    gateway: &Path,
    base_control_plane: &Path,
    realm: &Path,
) -> Result<()> {
    assert_executable(conformance)?;
    assert_executable(gateway)?;
    if !base_control_plane.is_file() {
        bail!(
            "base gateway control plane does not exist: {}",
            base_control_plane.display()
        );
    }
    if !realm.is_file() {
        bail!("Keycloak realm fixture does not exist: {}", realm.display());
    }

    let tmpdir = smoke_tmpdir()?;
    let mut cleanup = TmpDirGuard::new(tmpdir.clone());
    println!("smoke workspace: {}", tmpdir.display());

    let gateway_port = reserve_local_port()?;
    let keycloak_port = reserve_local_port()?;
    let gateway_base = format!("http://127.0.0.1:{gateway_port}");
    let keycloak_base = format!("https://127.0.0.1:{keycloak_port}");
    let issuer = format!("{keycloak_base}/realms/{KEYCLOAK_REALM}");
    let discovery_url = format!("{issuer}/.well-known/openid-configuration");
    let ca_path = tmpdir.join("keycloak-ca.pem");
    let cert_path = tmpdir.join("keycloak-cert.pem");
    let key_path = tmpdir.join("keycloak-key.pem");
    let control_plane = tmpdir.join("gateway.keycloak.json");
    let gateway_log = tmpdir.join("gateway.log");

    generate_keycloak_tls_material(
        &ca_path,
        &cert_path,
        &key_path,
        vec!["127.0.0.1".to_owned(), "localhost".to_owned()],
    )?;
    let suffix = uuid::Uuid::now_v7().simple().to_string();
    let container_name = format!("veoveo-keycloak-{suffix}");
    let _keycloak = ContainerGuard::new(container_name.clone());
    let realm = realm.canonicalize()?;
    let cert_path = cert_path.canonicalize()?;
    let key_path = key_path.canonicalize()?;
    run_checked(
        Path::new("docker"),
        [
            "run".into(),
            "--detach".into(),
            "--name".into(),
            container_name.clone().into(),
            "--publish".into(),
            format!("127.0.0.1:{keycloak_port}:8443").into(),
            "--env".into(),
            "KC_BOOTSTRAP_ADMIN_USERNAME=admin".into(),
            "--env".into(),
            "KC_BOOTSTRAP_ADMIN_PASSWORD=keycloak-ci-admin-password".into(),
            "--volume".into(),
            format!(
                "{}:/opt/keycloak/data/import/veoveo-ci-realm.json:ro",
                realm.display()
            )
            .into(),
            "--volume".into(),
            format!("{}:/opt/keycloak/conf/tls.crt:ro", cert_path.display()).into(),
            "--volume".into(),
            format!("{}:/opt/keycloak/conf/tls.key:ro", key_path.display()).into(),
            KEYCLOAK_IMAGE.into(),
            "start-dev".into(),
            "--import-realm".into(),
            "--http-enabled=false".into(),
            "--https-port=8443".into(),
            "--https-certificate-file=/opt/keycloak/conf/tls.crt".into(),
            "--https-certificate-key-file=/opt/keycloak/conf/tls.key".into(),
            format!("--hostname={keycloak_base}").into(),
        ],
        [],
    )?;

    let idp = keycloak_client(&ca_path)?;
    let discovery = wait_for_keycloak(&idp, &discovery_url, &container_name).await?;
    validate_keycloak_discovery(&discovery, &issuer)?;
    write_keycloak_control_plane(base_control_plane, &control_plane, &ca_path, &discovery)?;

    let auth_private_key = run_checked(conformance, ["gateway-private-key-der-b64".into()], [])?;
    let platform_store = spawn_gateway_platform_store(gateway, &control_plane).await?;
    let mut gateway_child = ChildGuard::spawn(
        gateway,
        gateway_serve_args(gateway_port, &platform_store),
        [
            (
                "VEOVEO_INTERNAL_SIGNING_KEY_DER_B64",
                INTERNAL_SIGNING_KEY_DER_B64.into(),
            ),
            (
                "VEOVEO_IDP_OIDC_CLIENT_SECRET",
                KEYCLOAK_CLIENT_SECRET.into(),
            ),
            (
                "VEOVEO_AUTHORIZATION_SERVER_PRIVATE_KEY_DER_B64",
                auth_private_key.trim().into(),
            ),
        ],
        &gateway_log,
    )?;
    wait_for_http(&format!("{gateway_base}/healthz")).await?;
    assert_ready_profiles(&gateway_base, 2).await?;

    exercise_keycloak_browser_flow(&gateway_base, &issuer, &ca_path).await?;

    gateway_child.stop();
    cleanup.remove_on_drop();
    println!("gateway Keycloak identity smoke ok ({KEYCLOAK_IMAGE})");
    Ok(())
}

pub fn generate_keycloak_tls_material(
    ca_cert_path: &Path,
    server_cert_path: &Path,
    server_key_path: &Path,
    sans: Vec<String>,
) -> Result<()> {
    let mut ca_params = rcgen::CertificateParams::default();
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "Veoveo Local Keycloak CA");
    let ca_key = rcgen::KeyPair::generate()?;
    let ca_cert = ca_params.self_signed(&ca_key)?;

    let mut server_params = rcgen::CertificateParams::new(sans)?;
    server_params.is_ca = rcgen::IsCa::ExplicitNoCa;
    server_params
        .extended_key_usages
        .push(rcgen::ExtendedKeyUsagePurpose::ServerAuth);
    let server_key = rcgen::KeyPair::generate()?;
    let ca_issuer = rcgen::Issuer::from_params(&ca_params, &ca_key);
    let server_cert = server_params.signed_by(&server_key, &ca_issuer)?;

    fs::write(ca_cert_path, ca_cert.pem())?;
    fs::write(server_cert_path, server_cert.pem())?;
    fs::write(server_key_path, server_key.serialize_pem())?;
    Ok(())
}

fn keycloak_client(cert_path: &Path) -> Result<reqwest::Client> {
    let cert = reqwest::Certificate::from_pem(&fs::read(cert_path)?)?;
    Ok(reqwest::Client::builder()
        .add_root_certificate(cert)
        .cookie_store(true)
        .redirect(Policy::none())
        .build()?)
}

async fn wait_for_keycloak(
    client: &reqwest::Client,
    discovery_url: &str,
    container_name: &str,
) -> Result<Value> {
    for _ in 0..240 {
        if let Ok(response) = client.get(discovery_url).send().await
            && response.status() == StatusCode::OK
            && let Ok(discovery) = response.json().await
        {
            return Ok(discovery);
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    let logs = run_raw(
        Path::new("docker"),
        ["logs".into(), container_name.into()],
        [],
    )?;
    bail!(
        "timed out waiting for Keycloak discovery at {discovery_url}\ncontainer logs:\n{}{}",
        String::from_utf8_lossy(&logs.stdout),
        String::from_utf8_lossy(&logs.stderr)
    );
}

fn validate_keycloak_discovery(discovery: &Value, expected_issuer: &str) -> Result<()> {
    let issuer = discovery_string(discovery, "issuer")?;
    if issuer != expected_issuer {
        bail!("Keycloak discovery issuer was {issuer}, expected {expected_issuer}");
    }
    for field in ["authorization_endpoint", "token_endpoint", "jwks_uri"] {
        let value = discovery_string(discovery, field)?;
        if !value.starts_with(expected_issuer) {
            bail!("Keycloak discovery `{field}` escaped the realm issuer: {value}");
        }
    }
    let supports_s256 = discovery
        .get("code_challenge_methods_supported")
        .and_then(Value::as_array)
        .is_some_and(|methods| methods.iter().any(|method| method == "S256"));
    if !supports_s256 {
        bail!("Keycloak discovery did not advertise PKCE S256: {discovery}");
    }
    let supports_client_secret_post = discovery
        .get("token_endpoint_auth_methods_supported")
        .and_then(Value::as_array)
        .is_some_and(|methods| methods.iter().any(|method| method == "client_secret_post"));
    if !supports_client_secret_post {
        bail!("Keycloak discovery did not advertise client_secret_post: {discovery}");
    }
    Ok(())
}

fn discovery_string<'a>(discovery: &'a Value, field: &str) -> Result<&'a str> {
    discovery
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("Keycloak discovery omitted `{field}`: {discovery}"))
}

pub(crate) fn remove_enterprise_managed_authorization(control_plane: &mut Value) -> Result<()> {
    let cp_obj = control_plane
        .as_object_mut()
        .context("gateway control plane root must be a JSON object")?;

    if let Some(idps) = cp_obj.get_mut("identity_providers") {
        let idps_arr = idps
            .as_array_mut()
            .context("`identity_providers` must be a JSON array")?;
        for idp in idps_arr {
            let idp_obj = idp
                .as_object_mut()
                .context("identity provider must be a JSON object")?;
            idp_obj.remove("enterprise_managed_authorization_endpoint");
        }
    }

    if let Some(profiles) = cp_obj.get_mut("profiles") {
        let profiles_arr = profiles
            .as_array_mut()
            .context("`profiles` must be a JSON array")?;
        for profile in profiles_arr {
            let profile_obj = profile
                .as_object_mut()
                .context("profile must be a JSON object")?;
            if let Some(auth_modes) = profile_obj.get_mut("auth_modes") {
                let auth_modes_arr = auth_modes
                    .as_array_mut()
                    .context("`auth_modes` must be a JSON array")?;
                auth_modes_arr.retain(|mode| mode != "enterprise_managed_authorization");
            }
        }
    }

    let mut removed_client_ids = std::collections::HashSet::new();
    if let Some(clients) = cp_obj.get_mut("oauth_clients") {
        let clients_arr = clients
            .as_array_mut()
            .context("`oauth_clients` must be a JSON array")?;

        let mut retained_clients = Vec::new();
        for mut client in clients_arr.drain(..) {
            let client_obj = client
                .as_object_mut()
                .context("oauth client must be a JSON object")?;
            let client_id = client_obj
                .get("id")
                .and_then(Value::as_str)
                .context("oauth client missing `id` string field")?
                .to_string();

            let grant_types_val = client_obj
                .get_mut("grant_types")
                .context("oauth client missing `grant_types`")?;
            let grant_types_arr = grant_types_val
                .as_array_mut()
                .context("oauth client `grant_types` must be a JSON array")?;

            grant_types_arr.retain(|grant| grant != "enterprise_managed_authorization");

            if grant_types_arr.is_empty() {
                removed_client_ids.insert(client_id);
            } else {
                retained_clients.push(client);
            }
        }
        *clients_arr = retained_clients;
    }

    if let Some(work_contexts) = cp_obj.get_mut("work_contexts") {
        let work_contexts_arr = work_contexts
            .as_array_mut()
            .context("`work_contexts` must be a JSON array")?;
        for wc in work_contexts_arr {
            let wc_obj = wc
                .as_object_mut()
                .context("work context must be a JSON object")?;
            if let Some(memberships) = wc_obj.get_mut("memberships") {
                let memberships_arr = memberships
                    .as_array_mut()
                    .context("`memberships` must be a JSON array")?;
                for membership in memberships_arr {
                    let membership_obj = membership
                        .as_object_mut()
                        .context("membership rule must be a JSON object")?;
                    if let Some(oauth_clients) = membership_obj.get_mut("oauth_clients") {
                        let oauth_clients_arr = oauth_clients
                            .as_array_mut()
                            .context("`oauth_clients` in membership must be a JSON array")?;
                        oauth_clients_arr.retain(|client_id_val| {
                            if let Some(id_str) = client_id_val.as_str() {
                                !removed_client_ids.contains(id_str)
                            } else {
                                true
                            }
                        });
                    }
                }
            }
        }
    }

    if let Some(clients) = cp_obj.get("oauth_clients")
        && let Some(clients_arr) = clients.as_array()
    {
        for client in clients_arr {
            let id = client
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>");
            let grants = client.get("grant_types").and_then(Value::as_array);
            if grants.is_none_or(|g| g.is_empty()) {
                bail!("OAuth client `{id}` has empty grant_types after adaptation");
            }
        }
    }

    if let Some(work_contexts) = cp_obj.get("work_contexts")
        && let Some(work_contexts_arr) = work_contexts.as_array()
    {
        for wc in work_contexts_arr {
            if let Some(memberships) = wc.get("memberships").and_then(Value::as_array) {
                for m in memberships {
                    if let Some(clients) = m.get("oauth_clients").and_then(Value::as_array) {
                        for c in clients {
                            if let Some(cid) = c.as_str()
                                && removed_client_ids.contains(cid)
                            {
                                bail!(
                                    "Work context membership retained reference to removed client `{cid}`"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn write_keycloak_control_plane(
    base: &Path,
    output: &Path,
    trusted_ca: &Path,
    discovery: &Value,
) -> Result<()> {
    let mut control_plane: Value = serde_json::from_str(&fs::read_to_string(base)?)?;
    remove_enterprise_managed_authorization(&mut control_plane)?;

    let identity_provider = control_plane
        .get_mut("identity_providers")
        .and_then(Value::as_array_mut)
        .and_then(|providers| {
            providers
                .iter_mut()
                .find(|provider| provider.get("id").and_then(Value::as_str) == Some("enterprise"))
        })
        .ok_or_else(|| anyhow!("control plane has no `enterprise` identity provider"))?;
    identity_provider["issuer"] = discovery["issuer"].clone();
    identity_provider["jwks"] = serde_json::json!({
        "source": "remote",
        "jwks_uri": discovery_string(discovery, "jwks_uri")?
    });
    identity_provider["authorization_endpoint"] = discovery["authorization_endpoint"].clone();
    identity_provider["token_endpoint"] = discovery["token_endpoint"].clone();
    identity_provider["claim_mapping"] = serde_json::json!({
        "subject": "sub",
        "tenant": {
            "claim": "tenant"
        }
    });
    identity_provider["trusted_certificate_authorities"] = serde_json::json!([
        {
            "source": "file",
            "path": trusted_ca.to_string_lossy()
        }
    ]);
    identity_provider["metadata"] = serde_json::json!({
        "provider": "keycloak",
        "realm": KEYCLOAK_REALM,
        "purpose": "provider_independent_identity_integration"
    });

    let oidc_client = control_plane
        .get_mut("oidc_clients")
        .and_then(Value::as_array_mut)
        .and_then(|clients| {
            clients
                .iter_mut()
                .find(|client| client.get("id").and_then(Value::as_str) == Some("enterprise"))
        })
        .ok_or_else(|| anyhow!("control plane has no `enterprise` OIDC client"))?;
    oidc_client["client_id"] = KEYCLOAK_CLIENT_ID.into();
    oidc_client["redirect_uri"] = format!("{PUBLIC_BASE_URL}/oauth/callback").into();
    oidc_client["auth_method"] = "client_secret_post".into();

    let parsed: veoveo_mcp_contract::GatewayControlPlane =
        serde_json::from_value(control_plane.clone())?;
    parsed.validate()?;
    fs::write(output, serde_json::to_vec_pretty(&control_plane)?)?;
    Ok(())
}

async fn exercise_keycloak_browser_flow(
    gateway_base: &str,
    issuer: &str,
    cert_path: &Path,
) -> Result<()> {
    let gateway_http = reqwest::Client::builder()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(15))
        .build()?;
    let browser = keycloak_client(cert_path)?;
    let client_id = "operator-local-public";
    let redirect_uri = "http://127.0.0.1:8789/oauth/callback";
    let code_verifier = "smoke-browser-pkce-verifier-0123456789abcdef0123456789abcdef";
    let code_challenge = "X9AgXux1PHu8RKlqHF9FuDYoLL6yjPFGS5je8BbaBF8";
    let client_state = "keycloak-smoke-state";
    let operator_resource = format!("{PUBLIC_BASE_URL}/mcp/operator");
    let authorize_query = form_urlencoded(&[
        ("response_type", "code"),
        ("client_id", client_id),
        ("redirect_uri", redirect_uri),
        ("scope", "operator:use"),
        ("resource", operator_resource.as_str()),
        ("code_challenge", code_challenge),
        ("code_challenge_method", "S256"),
        ("state", client_state),
    ]);
    let gateway_authorize = gateway_http
        .get(format!("{gateway_base}/oauth/authorize?{authorize_query}"))
        .send()
        .await?;
    let keycloak_authorize = redirect_location(gateway_authorize, StatusCode::FOUND)?;
    if !keycloak_authorize.starts_with(&format!("{issuer}/protocol/openid-connect/auth")) {
        bail!("gateway redirected to an unexpected identity endpoint: {keycloak_authorize}");
    }

    let login_page = browser.get(&keycloak_authorize).send().await?;
    if login_page.status() != StatusCode::OK {
        bail!(
            "Keycloak login page returned {}, expected 200",
            login_page.status()
        );
    }
    let login_html = login_page.text().await?;
    let document = Html::parse_document(&login_html);
    let selector = Selector::parse("form#kc-form-login")
        .map_err(|err| anyhow!("invalid Keycloak login selector: {err}"))?;
    let action = document
        .select(&selector)
        .next()
        .and_then(|form| form.value().attr("action"))
        .context("Keycloak login page omitted form#kc-form-login action")?;
    let action =
        reqwest::Url::parse(action).or_else(|_| reqwest::Url::parse(issuer)?.join(action))?;
    let login = browser
        .post(action)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(form_urlencoded(&[
            ("username", KEYCLOAK_USERNAME),
            ("password", KEYCLOAK_PASSWORD),
            ("credentialId", ""),
        ]))
        .send()
        .await?;
    let login_status = login.status();
    let idp_callback = match login_status {
        StatusCode::FOUND | StatusCode::SEE_OTHER => redirect_location(login, login_status)?,
        status => {
            let body = login.text().await.unwrap_or_default();
            bail!(
                "Keycloak login returned {status}; response body: {}",
                body.chars().take(1000).collect::<String>()
            );
        }
    };
    if !idp_callback.starts_with(&format!("{PUBLIC_BASE_URL}/oauth/callback")) {
        bail!("Keycloak returned an unexpected callback: {idp_callback}");
    }
    let callback_url = reqwest::Url::parse(&idp_callback)?;
    let callback_query = callback_url
        .query()
        .context("Keycloak callback omitted its query")?;
    let gateway_callback = gateway_http
        .get(format!("{gateway_base}/oauth/callback?{callback_query}"))
        .send()
        .await?;
    let client_redirect = redirect_location(gateway_callback, StatusCode::FOUND)?;
    if !client_redirect.starts_with(redirect_uri)
        || url_query_value(&client_redirect, "state")? != client_state
    {
        bail!("gateway returned an invalid browser callback: {client_redirect}");
    }
    let gateway_code = url_query_value(&client_redirect, "code")?;

    let token_response: Value = gateway_http
        .post(format!("{gateway_base}/oauth/token"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(form_urlencoded(&[
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("code", gateway_code.as_str()),
            ("redirect_uri", redirect_uri),
            ("code_verifier", code_verifier),
        ]))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let access_token = token_response
        .get("access_token")
        .and_then(Value::as_str)
        .context("authorization-code exchange omitted access_token")?;
    assert_keycloak_gateway_identity(access_token)?;
    let refresh_token = token_response
        .get("refresh_token")
        .and_then(Value::as_str)
        .context("authorization-code exchange omitted refresh_token")?;

    let rotated: Value = gateway_http
        .post(format!("{gateway_base}/oauth/token"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(form_urlencoded(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", refresh_token),
        ]))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let rotated_refresh_token = rotated
        .get("refresh_token")
        .and_then(Value::as_str)
        .context("refresh exchange omitted rotated refresh_token")?;
    if rotated_refresh_token == refresh_token {
        bail!("refresh exchange did not rotate the refresh token");
    }
    println!("Keycloak refresh token rotated");

    let duplicate_delivery = gateway_http
        .post(format!("{gateway_base}/oauth/token"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(form_urlencoded(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", refresh_token),
        ]))
        .send()
        .await?;
    println!(
        "immediate duplicate refresh returned {}",
        duplicate_delivery.status()
    );
    if duplicate_delivery.status() != StatusCode::OK {
        bail!(
            "immediate duplicate refresh returned {}, expected 200",
            duplicate_delivery.status()
        );
    }
    let duplicate_delivery: Value = duplicate_delivery.json().await?;
    let duplicate_refresh_token = duplicate_delivery
        .get("refresh_token")
        .and_then(Value::as_str)
        .context("duplicate refresh delivery omitted refresh_token")?;
    if duplicate_refresh_token != rotated_refresh_token {
        bail!("immediate duplicate refresh did not redeliver the identical successor");
    }
    println!("immediate duplicate refresh redelivered the identical successor");

    println!("waiting beyond the refresh duplicate-delivery window");
    tokio::time::sleep(Duration::from_secs(REFRESH_DELIVERY_WINDOW_SECONDS + 1)).await;
    let replay = gateway_http
        .post(format!("{gateway_base}/oauth/token"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(form_urlencoded(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", refresh_token),
        ]))
        .send()
        .await?;
    println!("delayed refresh replay returned {}", replay.status());
    if replay.status() != StatusCode::BAD_REQUEST {
        bail!(
            "refresh replay after the duplicate-delivery window returned {}, expected 400",
            replay.status()
        );
    }
    let replay_body: Value = replay.json().await?;
    if replay_body.get("error").and_then(Value::as_str) != Some("invalid_grant") {
        bail!("refresh replay did not return invalid_grant: {replay_body}");
    }
    let revoked_family = gateway_http
        .post(format!("{gateway_base}/oauth/token"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(form_urlencoded(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", rotated_refresh_token),
        ]))
        .send()
        .await?;
    println!(
        "successor after family revocation returned {}",
        revoked_family.status()
    );
    if revoked_family.status() != StatusCode::BAD_REQUEST {
        bail!(
            "delayed refresh replay did not revoke the rotated token family: {}",
            revoked_family.status()
        );
    }
    Ok(())
}

fn assert_keycloak_gateway_identity(access_token: &str) -> Result<()> {
    let payload = access_token
        .split('.')
        .nth(1)
        .context("gateway access token had no payload")?;
    let payload: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload)?)?;
    if payload.get("tenant").and_then(Value::as_str) != Some("tenant-a") {
        bail!("gateway token did not retain Keycloak tenant mapping: {payload}");
    }
    let has_operator_role = payload
        .get("roles")
        .and_then(Value::as_array)
        .is_some_and(|roles| roles.iter().any(|role| role == "operator"));
    if !has_operator_role {
        bail!("gateway token did not retain the Keycloak operator role: {payload}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_enterprise_only_client_and_work_context_references() -> Result<()> {
        let mut control_plane = serde_json::json!({
            "identity_providers": [
                {
                    "id": "enterprise",
                    "issuer": "https://idp.example.com",
                    "enterprise_managed_authorization_endpoint": "https://idp.example.com/oauth2/id-jag"
                }
            ],
            "profiles": [
                {
                    "id": "operator",
                    "auth_modes": [
                        "enterprise_managed_authorization",
                        "oidc_authorization_code_pkce"
                    ]
                }
            ],
            "work_contexts": [
                {
                    "id": "operations",
                    "memberships": [
                        {
                            "level": "contributor",
                            "oauth_clients": [
                                "operator-delegated",
                                "operator-local-public"
                            ]
                        }
                    ]
                }
            ],
            "oauth_clients": [
                {
                    "id": "operator-delegated",
                    "grant_types": [
                        "enterprise_managed_authorization"
                    ]
                },
                {
                    "id": "operator-local-public",
                    "grant_types": [
                        "authorization_code_pkce",
                        "refresh_token"
                    ]
                }
            ]
        });

        remove_enterprise_managed_authorization(&mut control_plane)?;

        // IdP endpoint removed
        assert!(
            control_plane["identity_providers"][0]
                .get("enterprise_managed_authorization_endpoint")
                .is_none()
        );

        // Profile auth_mode removed
        let auth_modes = control_plane["profiles"][0]["auth_modes"]
            .as_array()
            .unwrap();
        assert_eq!(
            auth_modes,
            &vec![serde_json::json!("oidc_authorization_code_pkce")]
        );

        // Exclusive enterprise client removed
        let clients = control_plane["oauth_clients"].as_array().unwrap();
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0]["id"], "operator-local-public");

        // Work context reference removed while ordinary client is preserved
        let wc_clients = control_plane["work_contexts"][0]["memberships"][0]["oauth_clients"]
            .as_array()
            .unwrap();
        assert_eq!(
            wc_clients,
            &vec![serde_json::json!("operator-local-public")]
        );

        Ok(())
    }

    #[test]
    fn preserves_mixed_grant_client_without_enterprise_grant() -> Result<()> {
        let mut control_plane = serde_json::json!({
            "identity_providers": [],
            "profiles": [],
            "work_contexts": [
                {
                    "id": "operations",
                    "memberships": [
                        {
                            "level": "contributor",
                            "oauth_clients": [
                                "mixed-client"
                            ]
                        }
                    ]
                }
            ],
            "oauth_clients": [
                {
                    "id": "mixed-client",
                    "grant_types": [
                        "authorization_code_pkce",
                        "enterprise_managed_authorization",
                        "refresh_token"
                    ]
                }
            ]
        });

        remove_enterprise_managed_authorization(&mut control_plane)?;

        let clients = control_plane["oauth_clients"].as_array().unwrap();
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0]["id"], "mixed-client");
        let grants = clients[0]["grant_types"].as_array().unwrap();
        assert_eq!(
            grants,
            &vec![
                serde_json::json!("authorization_code_pkce"),
                serde_json::json!("refresh_token")
            ]
        );

        // Work context reference preserved
        let wc_clients = control_plane["work_contexts"][0]["memberships"][0]["oauth_clients"]
            .as_array()
            .unwrap();
        assert_eq!(wc_clients, &vec![serde_json::json!("mixed-client")]);

        Ok(())
    }

    #[test]
    fn transformation_is_idempotent() -> Result<()> {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let smoke_cfg = repository.join("configs/gateway.smoke.json");
        let mut control_plane: Value = serde_json::from_str(&fs::read_to_string(&smoke_cfg)?)?;

        remove_enterprise_managed_authorization(&mut control_plane)?;
        let first_pass = control_plane.clone();

        remove_enterprise_managed_authorization(&mut control_plane)?;
        assert_eq!(control_plane, first_pass);

        // Ensure every remaining client has grants
        for client in control_plane["oauth_clients"].as_array().unwrap() {
            let grants = client["grant_types"].as_array().unwrap();
            assert!(
                !grants.is_empty(),
                "client {:?} has empty grants",
                client["id"]
            );
        }

        Ok(())
    }

    const REQUIRED_TEST_SANS: &[&str] = &["localhost", "127.0.0.1", "k3d-veoveo-keycloak"];

    fn required_test_sans() -> Vec<String> {
        REQUIRED_TEST_SANS
            .iter()
            .map(|san| san.to_string())
            .collect()
    }

    fn general_name_value(name: &x509_parser::extensions::GeneralName<'_>) -> Option<String> {
        use x509_parser::extensions::GeneralName;
        match name {
            GeneralName::DNSName(value) => Some((*value).to_owned()),
            GeneralName::IPAddress(bytes) => match bytes.len() {
                4 => Some(
                    std::net::Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]).to_string(),
                ),
                16 => <[u8; 16]>::try_from(*bytes)
                    .ok()
                    .map(std::net::Ipv6Addr::from)
                    .map(|address| address.to_string()),
                _ => None,
            },
            _ => None,
        }
    }

    #[test]
    fn generates_ca_and_server_material_in_a_tempdir() -> Result<()> {
        let workspace = tempfile::tempdir()?;
        let ca_cert_path = workspace.path().join("ca.pem");
        let server_cert_path = workspace.path().join("tls.crt");
        let server_key_path = workspace.path().join("tls.key");

        generate_keycloak_tls_material(
            &ca_cert_path,
            &server_cert_path,
            &server_key_path,
            required_test_sans(),
        )?;

        let ca_pem = x509_parser::pem::Pem::read(std::io::Cursor::new(fs::read(&ca_cert_path)?))
            .map_err(|error| anyhow!("reading generated CA PEM: {error}"))?
            .0;
        let ca_cert = ca_pem
            .parse_x509()
            .map_err(|error| anyhow!("parsing generated CA certificate: {error}"))?;
        assert!(
            ca_cert.is_ca(),
            "generated CA certificate must set basicConstraints CA:true"
        );

        let server_pem =
            x509_parser::pem::Pem::read(std::io::Cursor::new(fs::read(&server_cert_path)?))
                .map_err(|error| anyhow!("reading generated server certificate PEM: {error}"))?
                .0;
        let server_cert = server_pem
            .parse_x509()
            .map_err(|error| anyhow!("parsing generated server certificate: {error}"))?;
        assert!(
            !server_cert.is_ca(),
            "generated server certificate must not be a CA"
        );
        assert!(!fs::read_to_string(&server_key_path)?.is_empty());

        Ok(())
    }

    #[test]
    fn server_certificate_includes_required_subject_alternative_names() -> Result<()> {
        let workspace = tempfile::tempdir()?;
        let ca_cert_path = workspace.path().join("ca.pem");
        let server_cert_path = workspace.path().join("tls.crt");
        let server_key_path = workspace.path().join("tls.key");

        generate_keycloak_tls_material(
            &ca_cert_path,
            &server_cert_path,
            &server_key_path,
            required_test_sans(),
        )?;

        let server_pem =
            x509_parser::pem::Pem::read(std::io::Cursor::new(fs::read(&server_cert_path)?))
                .map_err(|error| anyhow!("reading generated server certificate PEM: {error}"))?
                .0;
        let server_cert = server_pem
            .parse_x509()
            .map_err(|error| anyhow!("parsing generated server certificate: {error}"))?;
        let observed = server_cert
            .subject_alternative_name()
            .context("reading generated server certificate SAN extension")?
            .context("generated server certificate omitted a Subject Alternative Name extension")?
            .value
            .general_names
            .iter()
            .filter_map(general_name_value)
            .collect::<std::collections::BTreeSet<_>>();
        let expected = required_test_sans().into_iter().collect();
        assert_eq!(observed, expected);

        Ok(())
    }

    #[test]
    fn loads_generated_ca_via_reqwest_and_rustls() -> Result<()> {
        let workspace = tempfile::tempdir()?;
        let ca_cert_path = workspace.path().join("ca.pem");
        let server_cert_path = workspace.path().join("tls.crt");
        let server_key_path = workspace.path().join("tls.key");

        generate_keycloak_tls_material(
            &ca_cert_path,
            &server_cert_path,
            &server_key_path,
            required_test_sans(),
        )?;

        let ca_bytes = fs::read(&ca_cert_path)?;
        reqwest::Certificate::from_pem(&ca_bytes)
            .context("loading generated CA into a reqwest trust root")?;

        let ca_der = x509_parser::pem::Pem::read(std::io::Cursor::new(ca_bytes))
            .map_err(|error| anyhow!("reading generated CA PEM: {error}"))?
            .0
            .contents;
        let mut roots = rustls::RootCertStore::empty();
        roots
            .add(rustls::pki_types::CertificateDer::from(ca_der))
            .map_err(|error| anyhow!("loading generated CA into a rustls trust store: {error}"))?;

        Ok(())
    }

    #[test]
    fn does_not_write_to_versioned_repository_paths() -> Result<()> {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let watched = [
            repository.join("showcase/sumo/deploy/ca.pem"),
            repository.join("configs/keycloak/tls.crt"),
            repository.join("configs/keycloak/tls.key"),
        ];
        let before = watched.iter().map(|path| path.exists()).collect::<Vec<_>>();

        let workspace = tempfile::tempdir()?;
        generate_keycloak_tls_material(
            &workspace.path().join("ca.pem"),
            &workspace.path().join("tls.crt"),
            &workspace.path().join("tls.key"),
            required_test_sans(),
        )?;

        let after = watched.iter().map(|path| path.exists()).collect::<Vec<_>>();
        assert_eq!(
            before, after,
            "generating Keycloak TLS material must not touch versioned repository paths"
        );
        Ok(())
    }
}
