# Enterprise deployment

Veoveo ships Kubernetes software as OCI images and Helm charts. The installation
owner supplies the cluster, registry access, configuration repository, secrets,
identity, ingress, and reconciliation controller. This boundary keeps a customer
installation recognizable to a Kubernetes platform team and prevents the product
repository from becoming the owner of customer infrastructure.

Helm is the package contract. GitOps is the recommended reconciliation model, with
Flux as the maintained reference. An operator may use another controller or direct Helm without
changing the chart, image, configuration, or Secret contracts.

The [enterprise discovery template](ENTERPRISE_DISCOVERY_TEMPLATE.md) records the
business and operating inputs that precede this contract. The
[enterprise installation runbook](ENTERPRISE_INSTALLATION_RUNBOOK.md) is the starting
point for delivery. It connects approved inputs to solution design, this technical
installation procedure, acceptance, and handoff. The
[enterprise installation readiness record](ENTERPRISE_INSTALLATION_READINESS.md)
captures the installation's decisions, evidence, defects, limitations, and approvals.

## Standards And Protocols

| Standard or protocol | Supported profile |
|---|---|
| OCI Distribution Specification | authenticated private image, chart, SBOM, provenance, schema, and evidence distribution |
| Helm and Kubernetes | separately reconciled platform and extension application charts |
| Flux 2.9.4 / GitOps Toolkit | maintained reference using `source.toolkit.fluxcd.io/v1`, `kustomize.toolkit.fluxcd.io/v1`, and `helm.toolkit.fluxcd.io/v2`; other controllers consume the same Helm and configuration contract |
| `veoveo.io/extension-release/v1` | independently published extension image, chart, fragment, conformance, and source identity |
| `veoveo.io/deployment/v6` | optional repository-development publication profile with exact platform selection, installation-owned Helm values, and managed GPU allocator closure |
| `veoveo.io/deployment-lock/v6` | immutable installation, source, and managed allocator evidence from the repository-development publication flow |
| `veoveo.io/gateway-server-fragment/v1` | extension-owned hosted-server contribution |
| `veoveo.io/gateway-binding/v1` | installation-owned exposure, tenant, producer, and authorization policy |
| `veoveo.io/compatibility-manifest/v1` | supported SDK, chart library, standalone tools, schemas, and optional simulation tuple |
| SHA-256 | production image, chart, schema, source-input, and evidence identity |
| OpenID Connect and OAuth 2.0 | installation-owned identity and protected-resource boundary |

## Installation Paths

Select one owner for application reconciliation. Do not operate the same resources
concurrently through more than one path.

| Path | Use when | Operational owner |
|---|---|---|
| GitOps | The organization reconciles Kubernetes desired state from Git | Flux or the organization's equivalent controller |
| Direct Helm | The organization has another controlled release process | The installation release process invoking Helm |
| Offline | The target cannot reach connected registries or package services | Connected bundle builder and offline installation operator |

Flux 2.9.4 is the maintained GitOps reference. Flux is not a Veoveo runtime dependency,
and Veoveo does not install or manage the organization's controller. Another controller
must preserve the same Helm, immutable source, ordering, readiness, and ownership
boundaries.

## Installation Inputs

Prepare these inputs before the first rollout:

1. A Kubernetes cluster with the storage, ingress, DNS, network egress, and GPU capacity
   required by the selected components.
2. A TLS-protected OCI registry reachable by publishers, controllers, and cluster nodes
   with their appropriate credentials.
3. Immutable Veoveo image identities and versioned Helm chart artifacts.
4. A private installation configuration repository or equivalently reviewed Direct Helm
   release input.
5. One canonical HTTPS installation origin.
6. OIDC/OAuth clients registered for that origin and the installation's selected scopes,
   roles, claims, and redirect URIs.
7. Every Kubernetes Secret referenced by the selected platform and extension charts.
8. The installation-owned gateway control plane, public trust files, bindings, and
   confidential Secret reference.
9. Storage classes, capacity, backup objectives, and restore procedures for SurrealDB,
   object storage, recordings, and selected persistent extensions.
10. An acceptance procedure for the installed platform and its selected business
    workflows.

The selected component graph determines the exact image, GPU, storage, Secret, and MCP
closure. Include only the components and dependencies approved for the installation.

## Ownership

| Concern | Owner | Durable source |
|---|---|---|
| Source compilation and image construction | Each source repository | Independent Git revision and repository-local image graph |
| Runtime images | OCI publisher | Registry manifests addressed by digest |
| Platform and extension packages | OCI publisher | Versioned Helm chart artifacts |
| Installation configuration | Installation owner | Private Git repository |
| Credentials and private keys | Installation owner | Secret manager and Kubernetes Secret projections |
| Cluster prerequisites | Installation platform team | Cluster platform repository |
| Application reconciliation | Installation GitOps controller | Declared source, Kustomization, and release objects |
| Acceptance evidence | Installation release process | Rust smoke, conformance, and operational evidence |

The [Autonomy Harness](AUTONOMY_HARNESS.md) defines the continuous containment boundary,
complete shared-responsibility matrix, and end-to-end operating proof for agents that
remain autonomous throughout the installation lifecycle. Helm readiness establishes
workload health, and the gateway independently probes each hosted server's declared
health endpoint so a degraded server surfaces in the Console; the harness evidence
proves that each effect stays inside its declared authority while agents keep running.

The build pipeline publishes artifacts. It does not connect to customer clusters.
The configuration repository selects published artifacts. It does not compile
Veoveo. The reconciliation controller reads the desired state and applies it to the
cluster. The smoke harness verifies the resulting installation without owning it.

## Installation Addressing

The installation owner chooses the canonical client-facing origin, for example
`https://veoveo.example.internal`. The name may resolve only through private DNS, an
internal load balancer, or a VPN. It remains distinct from the private OCI registry and
package-index coordinates.

`global.publicBaseUrl`, ingress hosts, OAuth protected resources, redirect URIs, and
gateway issuer metadata derive from that one installation-owned origin. No Veoveo
artifact embeds a universal service hostname, and private deployment does not require a
public Veoveo control plane.

Register the exact callback required by the gateway control plane with the selected
identity provider. Keep client credentials in the installation Secret system. Configure
NetworkPolicy and egress for issuer discovery, authorization, token, and JWKS endpoints.
Register only enterprise-approved identities, origins, certificates, users, and service
clients. Repository examples are not production dependencies.

Artifact downloads, Console downloads, and public-share bearers also remain on that
origin. RustFS or an external S3-compatible service is private installation
infrastructure. It has no client-facing ingress, DNS requirement, or presigned delivery
contract.

## Release Artifacts

One installation release may combine several independently published sources.
Production Helm values address images by digest; a mutable tag is not a production
identity. Each source builds, tests, certifies, and publishes its own image, chart,
gateway fragment, and release manifest.

The installation release procedure follows the
[external-repository runbook](EXTERNAL_REPOSITORY_INTEGRATION.md):

1. Verify the selected Veoveo compatibility manifest and extension-release manifests.
2. Check that every extension selects the installed compatibility release.
3. Pin image and chart digests in the installation's ordinary Helm values and GitOps
   release objects.
4. Compose installation-owned bindings with the selected gateway fragments.
5. Satisfy the generated typed platform requirements and render every chart.
6. Commit the complete desired-state change for normal reconciliation.

The Veoveo source provides a conventional private chart publisher:

~~~bash
REVISION=$(git rev-parse HEAD)
CHART_VERSION=0.1.0-$(git rev-parse --short=12 HEAD)

cargo xtask release helm-charts \
  --registry registry.example.internal/veoveo/charts \
  --version "$CHART_VERSION" \
  --revision "$REVISION"
~~~

An internal development registry may explicitly enable plain HTTP. A fielded registry
uses TLS, authentication, immutable tags, retention policy, and vulnerability scanning
supplied by the installation owner.

Veoveo's profile publisher remains available for development and source-publication
acceptance. The profile may live in a separate installation repository without making
`xtask` part of the fielded runtime or GitOps contract. It is documented in
[`LOCAL_DEPLOYMENT_PROFILES.md`](LOCAL_DEPLOYMENT_PROFILES.md) and is not required in an
installation repository.

Direct group publication remains available to Veoveo release pipelines. A Veoveo-backed
extension platform uses `external-extension-platform`; the canonical simulation base
and overlays use their dedicated groups. The simulation image group is an ABI and GPU
certification boundary, not proof that Frames, Map, Media, Optimization, or RRD
services were installed. The installation's typed chart selection and composed gateway
requirements supply that proof.

The canonical simulation runtime is a separate build dependency because UAV and
external simulator overlays consume it as a named build context. The deployment
profile derives the exact required platform targets and records their combined
immutable closure.

## Configuration Repository

An enterprise configuration repository should contain only installation-owned desired
state:

~~~text
clusters/
  production/
    platform/                 cluster prerequisites and controller configuration
    applications/             root and child reconciliation objects
    values/
      veoveo.yaml             installation identity, capacity, storage, ingress
      extension.yaml          independently deployed domain extension values
      images.yaml             reviewed image repositories and manifest digests
    releases/
      extension.json          selected immutable extension-release manifest
    gateway/
      base.json               installation-owned platform control plane
      bindings/               installation-owned extension exposure and policy
      fragments.lock.json     immutable extension fragment selection
      control-plane.json      deterministic composed output
      public-jwks.json
~~~

Helm values own chart inputs. Kubernetes manifests own resources outside a chart.
The gateway base and bindings own platform exposure and authorization. Extensions own
server fragments in their release artifacts. `gateway-compose` produces the complete
validated control plane and content provenance offline. The GitOps controller may
generate ConfigMaps from committed non-secret outputs. There is no second installation
document that repeats releases, values files, Secret keys, and apply order.

Environment overlays use the native composition mechanism chosen by the enterprise:
Helm values, Kustomize, or the GitOps controller's generator. One setting has one
canonical owner. A value is not copied into a general repository configuration file
merely because one installation needs it.

The Console public OAuth identity and its network route have separate installation
ownership. Keep `consoleBff.oauthResource` at the public protected-resource URL and set
`consoleBff.mcpTransportUrl` to the endpoint reachable by the BFF pod. Corporate roots
belong in a non-secret installation ConfigMap selected by
`consoleBff.outboundCa.existingConfigMap`; the chart mounts its configured PEM key and
the BFF adds those roots to the standard verifier. A deployment/v6 source lists the
owning values file under the platform release's `installationValues`. Missing ConfigMap
data blocks the pod mount, while malformed trust material blocks BFF startup.

## Secrets

Charts reference existing Kubernetes Secrets. Secret bytes never enter Helm values,
Git, a Flux Kustomization, or a generated ConfigMap. An enterprise may project those
Secrets with External Secrets Operator, Secrets Store CSI Driver, Sealed Secrets, or
its established platform mechanism.

The platform chart expects these Secret contracts by default:

| Secret | Required keys |
|---|---|
| veoveo-surreal-admin | username, password |
| veoveo-surreal-runtime | username, password |
| veoveo-installation-secrets | internal-signing-key-der-b64, internal-signing-key-id, internal-trust-jwks, oidc-client-secret, authorization-server-private-key-der-b64, refresh-delivery-key-b64, console-session-key, recording-playback-token-key, object-store-access-key, object-store-secret-key, media-provider-api-key, google-maps-api-key, media-provider-webhook-secret |

An extension declares its own least-privilege Secret references. It does not add
provider credentials to the platform Secret merely for convenience. Registry
credentials use a Kubernetes image pull Secret selected through Helm values.

Flux repository credentials are also platform Secrets. They authorize Flux to read
the enterprise Git and OCI repositories; they are not application credentials.

`recording-playback-token-key` is independent base64 text that decodes to exactly
32 random bytes. It signs only recording-scoped Redap read tokens and must not reuse a
gateway, refresh-delivery, Console session, object-store, or provider key.

For every credential, record its generator, public association, destination, minimum
permissions, rotation owner, overlap period, restart behavior, and rollback procedure.
Provision all referenced Secrets before starting dependent workloads. Never alternate
temporary and installation credentials as an in-place repair.

## Storage And Recovery

The platform chart runs one SurrealDB process backed by a RocksDB PVC. Database HA is
outside the current chart contract. Object storage and the Recording data plane also
hold durable installation state. Selected extensions may add independent single-writer
or persistent volumes.

Before rollout, choose storage classes and capacity and establish backup and restore
procedures that meet the approved recovery objectives. Exercise restore through the
organization's infrastructure procedure and verify application-level readability. A
successful Helm install or pod restart is not backup evidence.

## GitOps And Controller Boundary

The enterprise owns the GitOps controller. Veoveo applications must not install,
upgrade, configure, or delete that controller. The root Veoveo Kustomization begins only
after the controller and its repository credentials exist.

A root Kustomization may create the installation namespace, non-secret ConfigMaps,
ingress connectors, OCI sources, and HelmReleases. The platform chart is one release.
Each optional private MCP extension is another release with its own chart version,
values, health, rollback, and lifecycle.

The controller reconciles drift continuously. Routine releases change Git and let the
controller converge. kubectl apply and helm upgrade are bootstrap and recovery tools,
not concurrent owners of the same application resources.

The maintained Flux reference reconciles desired state in this order:

1. Namespace and installation-owned non-secret resources.
2. Secret projections and their readiness.
3. Git source and root Kustomization.
4. Immutable OCI chart sources.
5. Veoveo platform HelmRelease.
6. Independently deployed extension HelmReleases.
7. Installation-owned gateway activation and bindings.

The exact dependency expression belongs to the selected controller. The
[Bioma enterprise GitOps reference](../examples/bioma/README.md) demonstrates this
composition for one installation; its origins, identity, capacity, extensions, and
acceptance are not defaults. Its typed convergence command observes the Git artifact,
root Kustomization, release inventories, changed Deployments, and readiness:

~~~bash
cargo xtask smoke gitops-converge \
  --context <kubernetes-context> \
  --source <namespace/git-repository> \
  --root <namespace/root-kustomization> \
  --release <namespace/platform-helm-release> \
  --release <namespace/extension-helm-release> \
  --revision <full-git-revision> \
  --deployment <namespace/changed-deployment> \
  --evidence-output <new-evidence-path>
~~~

Pass every selected release and every Deployment changed by the rollout. The evidence
path is create-only.

## Independently deployed MCP extensions

An extension packages its Kubernetes workload in its own Helm chart. The installation
adds a HelmRelease for that chart, selects its immutable release manifest, and
binds its gateway fragment through installation-owned policy. The deterministic
composer registers routes and capabilities in the complete control plane. This
separates scheduling and rollout while preserving one MCP authority and one
authorization boundary.

An extension release normally selects two artifacts:

- the immutable OCI chart version;
- the installation Git repository containing values, bindings, and digest pins.

Every fragment's upstream declares two typed URLs: the MCP endpoint and a required
`health_url`. The Gateway Activation section governs their validation and health
semantics.

Private MCP servers follow the same pattern. They use their repository's native build
system and do not join the Veoveo workspace. Their chart consumes the versioned
`veoveo-extension` library from the configured private OCI registry or verified offline
bundle. Their gateway fragment still uses the canonical typed control-plane model,
internal assertion trust, policy checks, audit path, and URI identities. The complete
normative server requirements, including the well-known docs and contract resources,
are in
[`mcp/contract/DESIGN.md`](../mcp/contract/DESIGN.md).

## Gateway Activation

Compose and validate the complete gateway control plane before rollout. Installation
bindings own exposure, tenants, producers, profiles, scopes, and authorization policy;
extension fragments own their server contribution.

The installation must provide:

- the composed, validated control-plane document;
- public JWKS and required public CA material;
- the existing confidential Secret and its required keys;
- a content-addressed ConfigMap or equivalent immutable activation input consumed by
  the chart bootstrap path;
- one unauthenticated `health_url` for every hosted-server and Recording upstream.

The Gateway probes each declared health endpoint with `GET` and accepts only a success
status. An MCP response, authentication failure, or method rejection is not a health
signal. A fragment without `health_url` fails control-plane validation before rollout.

The Gateway may run multiple replicas because durable authority lives in the platform
store. Its Kubernetes Service uses client-IP affinity to keep each active MCP transport,
subscription, and notification stream attached to one Gateway process. Hosted MCP
server workloads retain their single-active-process lifecycle where required by their
runtime and storage contracts.

A GitOps controller may generate the ConfigMap from committed non-secret content while
preserving the chart contract. Never place confidential key bytes in that ConfigMap.

## Preflight

Before the first mutation in an environment, verify:

- every selected image and chart exists at its pinned immutable identity;
- cluster nodes can authenticate to and pull from the selected registry;
- all chart values render against the selected chart versions;
- every referenced Secret and required key is present;
- the gateway control plane validates and every upstream declares `health_url`;
- ingress, DNS, certificates, OIDC callbacks, and protected-resource origins agree;
- required storage classes and capacity exist;
- required NVIDIA GPU resources and runtime classes are allocatable;
- the controller can read the configuration and OCI repositories;
- the previous known-good release inputs and rollback procedure are available.

Use the chart's render and lint tools inside the installation release process. When a
repository deployment profile owns source-publication acceptance, its typed validation
and locked render provide corresponding evidence. Do not introduce a deployment profile
solely to replace an existing enterprise controller.

## Direct Helm

Flux is not a runtime dependency of Veoveo. An enterprise with another release
controller can render or install the same packages directly:

~~~bash
helm upgrade --install veoveo \
  oci://registry.example.internal/veoveo/charts/veoveo \
  --version "$CHART_VERSION" \
  --namespace veoveo \
  --create-namespace \
  --values values/veoveo.yaml \
  --values values/images.yaml \
  --wait
~~~

The operator must apply the gateway ConfigMap and provision every referenced Secret
before Helm starts workloads. Another GitOps system should express those same ordering
and ownership boundaries rather than translating them into a Veoveo-specific
orchestrator. Preserve the rendered release inputs and Helm result as release evidence.

## Offline Installation

Build the canonical bundle on a connected host, verify it at transfer boundaries, and
load it inside the offline environment. Follow the
[offline installation bundle](../deploy/offline/README.md) for the exact builder and
loader commands.

The bundle carries runtime images, chart material, schemas, checksums, image identities,
SBOMs, and public configuration. It excludes installation Secrets, TLS private keys,
internal OIDC configuration, site-specific trust, and site-approved model or TensorRT
engine files. Supply those inside the offline boundary before starting workloads.

The offline cluster still requires compatible NVIDIA drivers and the selected GPU
allocator and runtime. Loading images does not establish GPU readiness.

## Installation Acceptance

Helm or controller readiness is necessary but not sufficient. The installation gate
checks:

- controller health and exact desired-state revision;
- Git source and root Kustomization readiness at the same revision when GitOps is used;
- every selected release Ready with a non-empty inventory;
- pod and container readiness;
- persistent storage attachment;
- ingress and TLS at the canonical origin;
- OIDC discovery and authenticated human and machine flows;
- exact positive and negative Gateway capability catalogs;
- successful health probes for every hosted-server and Recording upstream;
- required GPU allocation and hardware execution;
- one approved business workflow for every installed workload;
- persistence, restart, backup, and restore behavior required by the accepted scope.

Use the acceptance harness owned by the selected installation. Evidence from another
installation or reference composition does not prove the selected services, identity,
network, data, or recovery objectives.

Record exact artifact identities, configuration revision, commands, results, and
environment in the installation release evidence. State explicitly when an acceptance
was not executed. The
[enterprise installation readiness record](ENTERPRISE_INSTALLATION_READINESS.md) keeps
static validation, runtime health, identity, policy, business effects, persistence,
restore, and clean reproduction as separate results.

## Upgrade And Rollback

A release change updates selected release manifests, chart versions, and image digests
in one reviewed commit.
Automated reconciliation may self-heal configuration drift, but promotion between
environments remains an explicit Git change. Rollback restores the previous known-good
manifests and digests. Database migration compatibility belongs to release notes and
must be evaluated before promotion.

Repeat the affected technical and business acceptance after promotion. Do not declare
success solely because new pods became Ready. Data migrations and storage restoration
may constrain rollback; resolve those constraints before promotion.

## Recovery And Diagnosis

Capture controller, release, Kubernetes, ingress, identity, storage, and workload state
before attempting repair. Determine which owner controls the failed boundary. Use only
the recovery procedures approved for the installation. Do not recreate a customer
cluster, replace identity infrastructure, or delete persistent resources as an ad hoc
repair.

When Kubernetes rejects immutable selector or StatefulSet changes, stop the rollout and
review the chart transition and data-retention consequences. Do not delete stateful
resources merely to make an upgrade pass. Record accepted product limitations without
changing core as part of an installation repair.

## Agent Workflow

Agents operating on an enterprise installation must:

1. Read this document, the Helm chart contract, the installation runbook, and the
   selected installation repository before acting.
2. Identify the reconciliation owner and never introduce a second concurrent owner.
3. Distinguish repository defaults, executable references, and installation-owned
   decisions. Reference values do not become general requirements.
4. Inspect rendered inputs and runtime evidence before proposing code or contract
   changes.
5. Treat missing installation configuration as an owner input, not an invitation to add
   a repository-wide validation rule.
6. Never create, reveal, copy, or commit customer Secret values.
7. Stop at a failed rollout step unless the installation owner authorizes the diagnosed
   recovery.
8. Report which technical and business acceptance paths ran and which remain unverified.
