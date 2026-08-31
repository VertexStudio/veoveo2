# Deployment guide

This is the operational entrypoint for installing Veoveo in an organization-owned
Kubernetes environment. It orders the repository's existing release, configuration,
identity, Secret, reconciliation, acceptance, upgrade, and recovery contracts. It does
not define new platform policy.

The normative ownership and artifact contract remains
[Enterprise deployment](ENTERPRISE_DEPLOYMENT.md). The Helm chart contract remains
[Veoveo Helm installation](../deploy/helm/veoveo/README.md). This guide explains how an
installation team applies those contracts without treating a reference topology as an
enterprise default.

An enterprise engagement starts with the
[discovery template](ENTERPRISE_DISCOVERY_TEMPLATE.md). After the organization approves
the initial workflow, owners, constraints, and acceptance outcomes, use the
[enterprise installation runbook](ENTERPRISE_INSTALLATION_RUNBOOK.md) to coordinate the
delivery stages around this detailed deployment procedure. Maintain the
[enterprise installation readiness record](ENTERPRISE_INSTALLATION_READINESS.md) as
the status and evidence ledger for the selected environment.

## Standards And Protocols

This workflow uses the standards and repository contracts already named by the
enterprise deployment contract:

| Boundary | Contract used by this guide |
|---|---|
| Runtime and chart distribution | OCI Distribution with immutable artifact identities |
| Workload installation | Kubernetes and Helm |
| Connected reconciliation | The installation owner's GitOps controller or direct Helm |
| Identity | Installation-owned OpenID Connect and OAuth 2.0 provider |
| Extension composition | Veoveo extension release, gateway fragment, binding, and compatibility manifests |
| Optional source-publication workflow | `veoveo.io/deployment/v6` and `veoveo.io/deployment-lock/v6` |
| Offline delivery | Checksummed bundle, image identities, chart, configuration schemas, and SPDX SBOMs |

The exact supported profiles are listed in
[Enterprise deployment](ENTERPRISE_DEPLOYMENT.md#standards-and-protocols).

## Choose The Installation Path

Veoveo has three fielded installation paths. Select one owner for application
reconciliation and do not operate the same resources concurrently with another path.

| Path | Use when | Operational owner |
|---|---|---|
| GitOps | The organization already reconciles Kubernetes desired state from Git | Flux or the organization's equivalent controller |
| Direct Helm | The organization has another controlled release process or performs an operator-managed installation | The installation release process invoking Helm |
| Offline | The target cannot reach connected registries or package services | Connected bundle builder and offline installation operator |

Flux 2.9.4 is the maintained GitOps reference. Flux is not a Veoveo runtime dependency,
and Veoveo does not install or manage the organization's controller. Another controller
must preserve the same Helm, immutable source, ordering, readiness, and ownership
boundaries.

## Ownership Before Installation

Assign the existing responsibility boundary before publishing or applying anything:

| Concern | Required owner |
|---|---|
| Kubernetes cluster, nodes, storage, networking, DNS, TLS, ingress, and GPU allocation | Installation platform team |
| OCI images, charts, SBOMs, provenance, and release evidence | Artifact publisher |
| Environment values, selected versions, gateway bindings, and desired state | Installation configuration repository |
| OIDC application and OAuth policy | Installation identity team |
| Secret generation, rotation, projection, and recovery | Installation Secret-management system |
| Application convergence | Selected GitOps or release controller |
| Technical and domain acceptance | Installation release process and domain owner |

The build pipeline publishes artifacts but does not connect to the customer cluster.
The configuration repository selects artifacts but does not compile Veoveo. The
reconciliation owner applies the desired state. Acceptance observes the resulting
installation without becoming its controller.

## Installation Inputs

Prepare these inputs before the first rollout:

1. A Kubernetes cluster with the storage, ingress, DNS, network egress, and GPU capacity
   required by the selected components.
2. A TLS-protected OCI registry reachable by the publisher, controller, and cluster
   nodes with their appropriate credentials.
3. Immutable Veoveo image identities and versioned Helm chart artifacts.
4. A private installation configuration repository or an equivalently reviewed direct
   Helm release input.
5. One canonical HTTPS installation origin.
6. An OIDC/OAuth client registered for that origin and the installation's selected
   scopes, roles, claims, and redirect URI.
7. Every Kubernetes Secret referenced by the selected platform and extension charts.
8. The installation-owned gateway control plane, public trust files, bindings, and
   confidential Secret reference.
9. Storage classes, capacity, backup objectives, and restore procedures for SurrealDB,
   object storage, recordings, and any selected persistent extension.
10. An acceptance procedure for the installed platform and its selected domain
    workloads.

The selected component graph determines the exact image, GPU, storage, Secret, and MCP
closure. Include only the components and dependencies approved for the installation.

## Canonical Origin And Identity

Choose one client-facing HTTPS origin, for example:

```text
https://veoveo.example.internal
```

Derive these installation-owned values from that origin:

- `global.publicBaseUrl`;
- ingress hosts and TLS certificates;
- OAuth protected-resource identifiers;
- registered browser redirect URIs;
- gateway issuer and endpoint metadata;
- Console-facing artifact, share, and recording routes.

The OCI registry and private object store are separate authorities. They do not become
browser-facing Veoveo origins.

Register the exact callback required by the gateway control plane with the selected
OIDC provider. Keep client credentials in the installation Secret system. Configure
network policy and egress for the provider's issuer, authorization, token, and JWKS
endpoints when the installation uses connected identity.

Register only enterprise-approved identities, origins, certificates, users, and service
clients. Repository examples are not production dependencies.

## Publish And Select Artifacts

Each source repository publishes its own immutable images, chart, gateway fragment,
release manifest, conformance evidence, and compatibility identity. The installation
selects those artifacts; it does not rebuild them during reconciliation.

The Veoveo repository publishes versioned charts with:

```bash
REVISION="$(git rev-parse HEAD)"
CHART_VERSION="0.1.0-$(git rev-parse --short=12 HEAD)"

cargo xtask release helm-charts \
  --registry registry.example.internal/veoveo/charts \
  --version "$CHART_VERSION" \
  --revision "$REVISION"
```

Replace the example registry with the installation's authenticated OCI destination.
Record the resulting chart version and image manifest digests in the configuration
repository. Production workload identity uses immutable digests rather than mutable
tags.

When independently published extensions are selected, follow the
[external repository integration runbook](EXTERNAL_REPOSITORY_INTEGRATION.md): verify
compatibility, pin the release, compose its gateway fragment with an
installation-owned binding, satisfy its generated platform requirements, and render
the combined desired state before promotion.

The repository's deployment profile publisher is available for development and
source-publication acceptance. It is not required in a fielded runtime or GitOps
repository.

## Configuration Repository

Keep installation-owned desired state in a private repository. The canonical layout is
described in [Enterprise deployment](ENTERPRISE_DEPLOYMENT.md#configuration-repository)
and separates:

- cluster prerequisites and controller configuration;
- Git sources, root Kustomizations, OCI sources, and release definitions;
- platform, extension, and immutable image values;
- selected release manifests;
- gateway base, bindings, locked fragments, composed output, and public trust files.

One setting has one owner. Do not duplicate an image identity, hostname, Secret value,
or policy across unrelated configuration surfaces.

The Bioma files under `examples/bioma/` are an executable reference for one
installation. Their Entra tenant, Cloudflare edge, hostname, capacity, GPU layout,
roles, scopes, and application selection are not neutral defaults.

## Secrets

Provision every referenced Kubernetes Secret before starting application workloads.
Secret bytes never belong in Helm values, Git, Flux Kustomizations or HelmReleases,
generated ConfigMaps, deployment locks, or acceptance evidence.

The platform's default Secret names and key contracts are listed in
[Enterprise deployment](ENTERPRISE_DEPLOYMENT.md#secrets). Selected extensions add
their own least-privilege Secret references. Registry and GitOps repository credentials
remain platform-controller Secrets rather than application credentials.

Use the organization's existing projection mechanism, such as External Secrets
Operator, Secrets Store CSI Driver, Sealed Secrets, or an equivalent managed process.
Generate independent values for signing, refresh delivery, Console sessions, recording
playback, object storage, and provider credentials according to the owning chart
contract. Repository examples never supply customer credentials.

## Storage And Recovery

The platform chart runs one SurrealDB process backed by a RocksDB PVC. Database HA is
outside the current chart contract. Object storage and the Recording data plane also
hold durable installation state.

Before rollout, the installation owner must choose storage classes and capacity and
establish backup and restore procedures that meet its recovery objectives. Exercise
restore using the organization's infrastructure procedure. A successful Helm install
or pod restart is not backup evidence.

Review extension charts separately for additional single-writer or persistent volumes.
For example, a selected analytical workspace may introduce its own storage lifecycle.

## Gateway Activation

Compose and validate the complete gateway control plane before rollout. Installation
bindings own exposure, tenants, producers, profiles, scopes, and authorization policy;
extension fragments own their server contribution.

The installation must provide:

- the composed, validated control-plane document;
- public JWKS and any required public CA material;
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

The profile-based repository workflow can prepare this activation for its supported
profiles. A normal enterprise controller may generate the ConfigMap from committed
non-secret content while preserving the same chart contract. Never place confidential
key bytes in that ConfigMap.

## Preflight

Before the first mutation in an environment, verify:

- every selected image and chart exists at its pinned immutable identity;
- cluster nodes can authenticate to and pull from the selected registry;
- all chart values render against the selected chart versions;
- every referenced Secret and required key is present;
- the gateway control plane validates, every upstream declares `health_url`, and its
  installation-owned files exist;
- ingress, DNS, certificates, OIDC callbacks, and protected-resource origins agree;
- required storage classes and capacity exist;
- required NVIDIA GPU resources and runtime classes are allocatable;
- the controller can read the configuration and OCI repositories;
- the previous known-good release inputs and rollback procedure are available.

Use the chart's normal render and lint tools inside the installation's release process.
When a repository deployment profile owns the composition, its typed validation and
locked render provide the corresponding preflight. Do not invent a profile solely to
replace an existing enterprise controller.

## GitOps Installation

Provision the organization's controller and its repository credentials outside the
Veoveo application lifecycle. The maintained Flux reference reconciles installation
desired state in this order:

1. Namespace and installation-owned non-secret resources.
2. Secret projections and their readiness.
3. Git source and root Kustomization.
4. Immutable OCI chart sources.
5. Veoveo platform HelmRelease.
6. Independently deployed extension HelmReleases.
7. Installation-owned gateway activation and bindings as declared by the selected
   configuration layout.

The exact dependency expression belongs to the selected controller. Routine releases
change reviewed Git state and let that controller converge. Do not alternate `kubectl
apply`, direct `helm upgrade`, and GitOps reconciliation as concurrent owners of the
same resources.

Use [Bioma enterprise GitOps reference](../examples/bioma/README.md) as an executable
Flux example while substituting the installation's own origins, identity, secrets,
capacity, extensions, and acceptance. Its typed convergence command observes the exact
Git artifact, root Kustomization, Helm release inventories, changed Deployments, and
readiness:

```bash
cargo xtask smoke gitops-converge \
  --context <kubernetes-context> \
  --source <namespace/git-repository> \
  --root <namespace/root-kustomization> \
  --release <namespace/platform-helm-release> \
  --release <namespace/extension-helm-release> \
  --revision <full-git-revision> \
  --deployment <namespace/changed-deployment> \
  --evidence-output <new-evidence-path>
```

Pass every selected release and every Deployment changed by the rollout. The evidence
path is create-only.

## Direct Helm Installation

Direct Helm consumes the same chart, values, images, Secrets, and gateway activation as
GitOps. Apply the installation-owned gateway ConfigMap and ensure every referenced
Secret exists before starting workloads.

```bash
helm upgrade --install veoveo \
  oci://registry.example.internal/veoveo/charts/veoveo \
  --version "$CHART_VERSION" \
  --namespace veoveo \
  --create-namespace \
  --values values/veoveo.yaml \
  --values values/images.yaml \
  --wait
```

The paths and registry are placeholders for the installation configuration. Preserve
the rendered release inputs and Helm result as part of its release evidence.

## Offline Installation

Build the canonical bundle on a connected host, verify it at transfer boundaries, and
load it inside the offline environment. Follow
[Offline installation bundle](../deploy/offline/README.md) for the exact builder and
loader commands.

The bundle carries runtime images, chart material, schemas, checksums, image identities,
SBOMs, and public configuration. It deliberately excludes installation Secrets, TLS
private keys, internal OIDC configuration, site-specific trust, and site-approved model
or TensorRT engine files. Supply those inside the offline boundary before starting
workloads.

The offline cluster still requires compatible NVIDIA drivers and the selected GPU
allocator/runtime. Loading images does not establish GPU readiness.

## Installation Acceptance

Helm or controller readiness is necessary but not sufficient. The existing enterprise
gate checks:

- controller health and exact desired-state revision;
- Git source and root Kustomization readiness at the same revision;
- every selected HelmRelease Ready with a non-empty inventory;
- pod and container readiness;
- persistent storage attachment;
- ingress and TLS at the canonical origin;
- OIDC discovery and an authenticated browser or machine-client flow;
- MCP capability discovery through the gateway;
- successful health probes for every hosted-server and Recording upstream;
- required GPU allocation and hardware execution;
- domain acceptance for every installed workload.

Use the acceptance harness owned by the selected installation. Evidence from another
installation or reference composition does not prove the selected services, identity,
network, data, or recovery objectives.

Record the exact artifact identities, configuration revision, commands, results, and
environment in the installation release evidence. State explicitly when an acceptance
was not executed.

Use the readiness record to keep static validation, deployed runtime health, identity,
policy, business effects, persistence, restore, and clean reproduction as separate
results. Record accepted product limitations without changing core as part of an
installation repair.

## Upgrade And Rollback

An upgrade changes selected release manifests, chart versions, image digests, gateway
composition, and installation values through one reviewed desired-state transition.
Evaluate database migration compatibility and release notes before promotion.

Observe convergence through the selected controller and repeat technical and domain
acceptance. Do not declare success solely because new pods became Ready.

Rollback restores the previous known-good manifest, chart, image, gateway, and values
identities. Data migrations and storage restoration may constrain rollback; resolve
those constraints before promotion rather than discovering them during an incident.

## Recovery And Diagnosis

Capture controller, release, Kubernetes, ingress, identity, storage, and workload state
before attempting repair. Determine which owner controls the failed boundary.

Use only the recovery procedures approved for the installation. Do not recreate a
customer cluster, replace identity infrastructure, or delete persistent resources as an
ad hoc repair.

When Kubernetes rejects immutable selector or StatefulSet changes, stop the rollout and
review the chart/release transition and data-retention consequences. Do not delete
stateful resources merely to make an upgrade pass.

## Agent Workflow

Agents operating on an enterprise deployment must:

1. Read this guide, [Enterprise deployment](ENTERPRISE_DEPLOYMENT.md), the Helm chart
   contract, and the selected installation repository before acting.
2. Identify the reconciliation owner and never introduce a second concurrent owner.
3. Distinguish repository defaults, executable references, and installation-owned
   decisions. Reference values do not become general requirements.
4. Inspect rendered inputs and runtime evidence before proposing code or contract
   changes.
5. Treat missing installation configuration as an owner input, not an invitation to add
   a repository-wide validation rule.
6. Never create, reveal, copy, or commit customer Secret values.
7. Stop at the first failed rollout step unless the installation owner authorizes the
   diagnosed recovery.
8. Report which technical and domain acceptance paths actually ran and which remain
   unverified.
