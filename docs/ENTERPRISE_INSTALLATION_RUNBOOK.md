# Enterprise Installation Runbook

This runbook governs an AI-guided Veoveo enterprise delivery from the first customer
conversation through operational handoff. The installation agent uses it to choose the
next question, discover authorized facts, prepare bounded plans, request approvals,
execute technical stages, and preserve evidence. Detailed commands and deployment
contracts remain in the linked owning documents.

The customer begins by describing the organization and desired outcome in ordinary
language. They do not need to read this runbook or complete every installation document
first. Every identity, origin, policy, credential, production change, and recovery
procedure still requires approval from its responsible enterprise owner.

## Standards And Protocols

The installation boundary may include OCI images and charts, Helm, Kubernetes, a GitOps
controller, OpenID Connect, OAuth 2.0, private DNS, TLS, an enterprise Secret projection
mechanism, and Veoveo's versioned deployment, extension, compatibility, gateway, and
evidence contracts. The selected versions and supported profiles must be recorded in
the installation configuration and release evidence.

This runbook does not imply that every enterprise must use Flux, public networking, a
specific cloud, or a repository-development deployment profile. Helm is the package
contract. The enterprise owns its controller and infrastructure choices.

## Start With A Conversation

A request such as the following is enough to begin:

> Help us install Veoveo for our organization. Guide us through the information and
> approvals you need.

The installation agent responds with the first business question. It does not answer
with a reading list, a complete questionnaire, a manifest, or a cluster command. The
first exchange establishes:

- the organization and business area;
- the first outcome the organization wants to improve;
- the person accountable for that outcome;
- whether this is exploration, planning, or an authorized installation engagement.

When one of these facts is unknown, the agent records it as an open question with an
owner. It does not invent an answer or block unrelated discovery.

## Guided Conversation Contract

The agent repeats one controlled loop throughout the engagement:

1. **Orient.** State the current stage, known facts, and the gate being approached.
2. **Ask.** Request the smallest coherent set of owner decisions needed for that gate.
3. **Discover.** With authorization, inspect facts the environment can provide directly.
4. **Distinguish.** Mark each value as observed fact, enterprise decision, product
   contract, assumption, or unresolved question.
5. **Record.** Update the installation readiness record without storing Secret values.
6. **Propose.** Present the next bounded plan, expected effect, evidence, and recovery
   boundary.
7. **Approve or advance.** Obtain the required owner approval before mutation, then run
   only the approved step and report its result.

Do not ask every discovery question at once. Use the question bank in
[Enterprise discovery](ENTERPRISE_DISCOVERY_TEMPLATE.md) according to the current gate:

| Conversation stage | Begin with | Advance when |
|---|---|---|
| Outcome | What should become possible, safer, faster, or more reliable? | One bounded result and business owner are named |
| Workflow | Who acts, what decision is made, and what observable result proves success? | Current and intended paths, effects, and acceptance owner are understood |
| Authority | Who may read, propose, approve, execute, administer, and audit? | Human and machine boundaries have owners |
| Systems | Which existing systems and data are required for the first workflow? | Interfaces, owners, identifiers, and permitted effects are known |
| Installation | Where will it run and who owns identity, network, storage, Secrets, and reconciliation? | Architecture decisions and unresolved prerequisites have owners |
| Operation | Who monitors, recovers, upgrades, and accepts residual risk? | Handoff, recovery, and final approval responsibilities are explicit |

The agent reuses answers already supplied. It asks a specialist only for decisions owned
by that specialist and explains why the answer is needed. It may group closely related
questions, but each group must be short enough for the customer to answer confidently.

## Authority And Approval

Conversation grants no infrastructure authority by itself. The agent operates in one of
four explicit modes:

| Mode | Permitted behavior |
|---|---|
| `discover` | Ask questions and perform approved read-only inspection |
| `plan` | Prepare configuration changes, commands, acceptance procedures, and recovery boundaries without applying them |
| `execute` | Apply the exact approved bounded change and collect sanitized evidence |
| `recover` | Perform an owner-approved recovery after preserving the causal failure and protected-resource inventory |

Require an accountable owner before:

- creating or changing identity clients, roles, policies, or service authority;
- handling or projecting credentials and private material;
- changing public DNS, TLS, ingress, firewall, proxy, or external exposure;
- reconciling a fielded environment or promoting a production release;
- modifying persistent schemas, retention, backup, restore, or protected data;
- rolling back, deleting, recreating, or taking another destructive recovery action;
- accepting a product limitation, failed check, or residual business risk.

An approval names the plan or command scope, environment, expected effect, recovery
boundary, and approver. General encouragement to continue is not approval for an
unbounded or destructive action.

## Installation Memory

The agent creates one installation-specific copy of
[Enterprise installation readiness](ENTERPRISE_INSTALLATION_READINESS.md) as soon as the
organization, environment, and installation owner can be identified. That record is the
durable state of the engagement. After every decision or check, the agent updates:

- the current stage and next gate;
- confirmed decisions and their owners;
- facts observed directly and their evidence locations;
- open questions, assumptions, dependencies, and blockers;
- approved plans and the scope of each approval;
- commands or procedures executed and sanitized results;
- failed, blocked, not-executed, or accepted-limit checks without converting them to
  success;
- the next question, approval, or executable step.

Chat history and shell history are not installation memory. The readiness record links
to enterprise-owned configuration and evidence repositories instead of copying Secret
values or sensitive payloads.

## Source Documents

The installation agent consults these sources as the workflow requires them:

- [Veoveo explained](VEOVEO_OVERVIEW.md) for the non-technical product model;
- [Enterprise discovery template](ENTERPRISE_DISCOVERY_TEMPLATE.md) for engagement
  inputs and decision ownership;
- [Enterprise installation readiness](ENTERPRISE_INSTALLATION_READINESS.md) for the
  installation's decision, prerequisite, acceptance, defect, and residual-risk record;
- [Enterprise deployment](ENTERPRISE_DEPLOYMENT.md) for the complete technical contract
  and procedure, including artifacts, configuration, Secrets, GitOps, Direct Helm,
  offline installation, Gateway activation, acceptance, upgrade, and recovery;
- [External repository integration](EXTERNAL_REPOSITORY_INTEGRATION.md) for private
  extension publication and composition;
- [Autonomy harness](AUTONOMY_HARNESS.md) when agents will operate continuously;
- the selected Helm chart and component design documents.

The customer does not need to navigate these sources during the guided conversation.
An installation-specific repository becomes the durable source for customer decisions.
This product repository remains the source for Veoveo contracts and release artifacts.

## Operating Principles

1. One owner reconciles each resource. Do not alternate Helm, kubectl, scripts, and
   GitOps against the same object without an explicit recovery procedure.
2. One canonical origin drives public URLs, OAuth resources, redirect URIs, and issuer
   metadata.
3. One approved Secret authority supplies each credential. Temporary or example
   material never overwrites fielded material.
4. Images and charts are selected by immutable identity.
5. Configuration is public desired state; Secret values remain in the enterprise
   Secret system.
6. Platform readiness, identity acceptance, authorization, domain behavior, and
   recovery are separate gates.
7. Capture evidence before repair. A failed customer composition does not automatically
   authorize a core product change.
8. Stop before destructive recovery, data migration, authority expansion, or an
   ownership change unless the responsible enterprise owner approves it.
9. Classify installation composition, infrastructure prerequisites, acceptance
   procedures, enterprise extensions, and core product behavior separately.
10. Preserve failed and blocked results. A known product limitation may be accepted for
    a bounded workflow, but it must not be hidden or repaired through an undocumented
    state rewrite.

## Delivery Record

The installation agent creates this delivery record before implementation and maintains
it from owner-approved answers and observed facts.

| Field | Value |
|---|---|
| Organization and installation | |
| Environment | |
| Business owner | |
| Installation owner | |
| Security approver | |
| Operations owner | |
| Configuration repository | |
| Approved source revision | |
| Compatibility release | |
| Change or release identifier | |
| Canonical origin | |
| Maintenance window | |
| Rollback decision owner | |
| Evidence location | |
| Readiness record | |

## Stage 0: Discovery Approval

### Inputs

- guided discovery state, including open questions and their owners;
- bounded first business workflow;
- identified systems and data owners;
- named identity, security, infrastructure, and operations owners;
- preliminary technical and business acceptance scenarios.

### Actions

1. Conduct the progressive discovery conversation using the discovery question bank.
2. Confirm what Veoveo will provide and what the enterprise will provide.
3. Separate the first release from later integrations.
4. Identify prohibited data, actions, destinations, and operating modes.
5. Record unresolved assumptions with owners and due dates.
6. Choose the solution-design approvers.

### Gate

Do not begin installation design when the business outcome, authority boundary, or
installation owner is unknown. Continue guided discovery instead of sending the customer
away to complete the template alone.

## Stage 1: Solution And Ownership Design

### Define The Composition

Record the selected:

- platform components;
- first-party MCP services;
- enterprise-owned or supplier-owned extensions;
- user-facing applications;
- Work Contexts and tenants;
- human roles and machine identities;
- read, propose, approve, execute, administer, and audit boundaries;
- data flows, artifacts, recordings, and retention behavior.

### Assign Owners

| Concern | Owner | Durable source | Reconciliation mechanism |
|---|---|---|---|
| Cluster prerequisites | | | |
| OCI registry and artifact policy | | | |
| Installation Git repository | | | |
| GitOps controller or release process | | | |
| Identity provider configuration | | | |
| Secret generation and projection | | | |
| Gateway base and bindings | | | |
| Platform release | | | |
| Each extension release | | | |
| Persistent storage and object storage | | | |
| Backup and recovery | | | |
| Monitoring and incident response | | | |
| Acceptance evidence | | | |

### Gate

Each resource must have one reconciliation owner. Resolve overlaps before manifests or
scripts are written.

## Stage 2: Installation Architecture

### Environment Decisions

Document:

- Kubernetes distribution, version, architecture, and support owner;
- connected, restricted-egress, or offline operating mode;
- node pools, NVIDIA GPU inventory, drivers, runtime, and allocation mechanism;
- private registry, authentication, mirrors, retention, and scanning;
- Git service and reconciliation controller;
- private DNS, canonical origin, ingress, certificates, and trust roots;
- identity provider, issuer, clients, claims, groups, and service identities;
- network segmentation, allowed ingress, egress, proxies, and private endpoints;
- storage classes, object storage, encryption, capacity, backup, and restore;
- node and image-filesystem thresholds, eviction policy, image garbage collection, and
  the operating margin required for upgrades;
- monitoring, logging, audit export, alerting, and support routing.

### Capacity

Estimate workload replicas, CPU, memory, GPU, storage, object growth, recording rate,
network traffic, image footprint, and retention. State expected growth and the point at
which capacity must be reviewed.

Inventory transitive runtime images, including init containers, Jobs, the Kubernetes
sandbox image, GPU components, databases, and administrative utilities. Prove that
cluster nodes can resolve and pull them through the selected connected, mirrored, or
offline path. A publisher-host image cache is not node-runtime evidence.

### Gate

The platform team must prove prerequisites independently of Veoveo workloads. A Helm
render cannot prove DNS, identity, storage performance, GPU execution, or backup
capability.

## Stage 3: Artifact Selection And Publication

### Select Releases

1. Select a supported Veoveo compatibility release.
2. Select each independently published extension release.
3. Verify source revision, chart version, image digest, schema, SBOM, provenance, and
   conformance evidence as required by enterprise policy.
4. Verify extension compatibility with the selected platform release.
5. Mirror or transfer artifacts when the destination cannot use the publisher's
   registry directly.

### Record Immutable Inputs

| Artifact | Source revision | OCI reference | Digest or version | Verification |
|---|---|---|---|---|
| Platform chart | | | | |
| Platform images | | | | |
| Extension chart | | | | |
| Extension images | | | | |
| Compatibility manifest | | | | |
| Gateway fragments | | | | |
| Offline bundle, if used | | | | |

### Gate

Do not promote mutable tags as production identity. Do not build customer extensions
inside the customer cluster unless that process is an explicitly designed and approved
enterprise build service.

## Stage 4: Installation Repository

Create or update the enterprise-owned desired-state repository. Follow the structure
in [Enterprise deployment](ENTERPRISE_DEPLOYMENT.md).

The reviewed change should contain:

- namespaces and non-secret prerequisites owned by the installation;
- chart sources and immutable release selections;
- environment-specific Helm values;
- image repositories and digests;
- gateway base, extension fragments, and installation-owned bindings;
- ingress, DNS integration, certificates references, and trust ConfigMaps;
- Secret references, never Secret values;
- storage and backup selections;
- monitoring and policy resources;
- acceptance configuration and release metadata.

Render and validate the complete composition before reconciliation. Check that every
referenced Secret, ConfigMap, service, route, health endpoint, storage claim, and image
exists in the intended ownership boundary.

Record transport URLs, HTTP authorities, public OAuth resources, audiences, issuers,
redirect URIs, and TLS names independently. They often share a hostname, but they are
not interchangeable contract fields.

### Gate

The installation repository must contain one coherent desired state. Do not rely on an
operator remembering an unrecorded kubectl patch after reconciliation.

## Stage 5: Secret And Identity Readiness

### Secret Inventory

| Secret contract | Keys or fields | Generator | Store | Projection owner | Rotation owner |
|---|---|---|---|---|---|
| | | | | | |

For every credential:

- identify the authoritative generator;
- identify the public material or remote registration it must match;
- confirm its destination and minimum permissions;
- define rotation, overlap, restart, and rollback behavior;
- prove that logs, rendered manifests, evidence, and Git do not contain its value.

### Identity Readiness

Verify the canonical issuer, discovery, clients, audiences, redirect URIs, claims,
groups, roles, service identities, logout, session, and administrative ownership.

Use separate human and machine acceptance. A working administrator login does not
prove an operator's policy or a service client's assertion.

For signing identities, verify that each private key matches its registered public key
and key identifier. Record the association and algorithm without exposing private
material. Validate service clients from their real calling namespace and NetworkPolicy,
including DNS, identity, Gateway, storage, and domain-specific egress.

### Ordering Rule

Provision the approved Secret contracts and identity registrations before workloads
that consume them. If a Secret changes after a pod starts, follow the documented
component rollout procedure. Do not alternate temporary credentials and installation
credentials as a repair technique.

### Gate

Run a non-secret closure check. Every selected release must resolve its Secret and
ConfigMap references, and matching public/private identity pairs must be validated
without printing private material.

## Stage 6: Reconciliation

### Recommended GitOps Path

1. Confirm the controller and repository credentials are healthy.
2. Commit the reviewed desired-state revision.
3. Reconcile cluster prerequisites in their owned order.
4. Reconcile identity and Secret projections.
5. Reconcile the platform release.
6. Reconcile independently deployed extensions.
7. Activate the complete composed gateway control plane.
8. Wait for declared health and readiness gates.

### Direct Helm Path

Use the ordered Direct Helm procedure in [Enterprise deployment](ENTERPRISE_DEPLOYMENT.md#direct-helm) when
the enterprise selected that release model. Preserve rendered inputs and Helm results.
Do not leave a GitOps controller and an operator concurrently managing the same
release.

### Stop Conditions

Stop and capture evidence when:

- an image or chart identity differs from the approved release;
- a Secret or public-key association fails;
- an immutable resource change has data-retention consequences;
- a migration reports an error or unknown state;
- a stateful workload cannot attach its expected storage;
- identity, gateway, or extension health fails;
- a required GPU workload lacks hardware-backed execution;
- the proposed repair changes infrastructure or authority ownership.

An API-server `/readyz` result does not satisfy application readiness. Check desired and
Ready replicas, current ReplicaSet ownership, pod conditions, restarts, effective image
IDs, Jobs, storage attachment, node pressure, image pulls, and dependency health.

## Stage 7: Layered Acceptance

Run each layer separately and record failures as failures. Do not let one green layer
stand in for another.

### Release And Infrastructure

- exact desired-state revision and artifact identities;
- controller and release health;
- pods, containers, jobs, services, and endpoints;
- PVC attachment, object storage, and database readiness;
- ingress, DNS, TLS, and enterprise trust;
- required NVIDIA GPU allocation and hardware execution.

### Identity And Policy

- OIDC discovery and human login;
- service-identity authentication;
- expected tenant, Work Context, role, and scope claims;
- positive capability catalog for each role;
- negative catalog and action checks;
- privileged administration separated from ordinary work.

### Service Contracts

- gateway health for every selected upstream;
- MCP capability and documentation discovery;
- representative read-only request;
- bounded mutation when the business workflow requires it;
- idempotency and ambiguous-failure behavior;
- audit and durable task or work state.

For MCP services, record the negotiated protocol version, metadata required on every
request, exact human and machine capability catalogs, and representative denials.
Measure real response shapes and sizes. Model full responses deliberately when an API
returns totals with truncated detail collections.

For MCP Apps, verify the caller-visible `ui://` catalog, linked tools, sandbox and CSP,
reactive-resource behavior, and any projected cross-server resource dependency. A
partial service failure may degrade its own App catalog, but it must not hold the entire
Console in a loading state. Test the dependency through the composed Gateway profile;
the App's browser message cannot enlarge the server, URI prefix, operation, scope, or
data labels declared by the installation.

### Business Workflow

Run the accepted scenario from the public enterprise boundary. Confirm the initial
state, authorization decision, external effect, resulting state, evidence correlation,
and user-visible result. Use real domain evidence rather than inference from pod health.

If the workflow mutates an external system, reserve a durable execution before the
effect. Define identical and conflicting replay behavior, failure after the effect, and
the owner of ambiguous recovery. Use an exact domain observation rather than a
probabilistic metric.

### Data And Recovery

- artifact or recording access through governed routes;
- backup creation and independent restore validation;
- restart persistence without deleting storage;
- upgrade and rollback within the approved compatibility boundary;
- monitoring and alert delivery to the enterprise owner.

When recordings are part of the workflow, prove governed catalog discovery, exact
selection, durable correlation to the business execution, Live consumption,
finalization, sealing, and History separately. Temporal proximity is not correlation,
and a Live recording is not evidence of a sealed History manifest.

Record the selected release's durable dataset UUID, recording UUID, layer identities,
catalog revision, grant class, Blueprint source, and playback-manifest schema. Verify
that immutable RRD layers are published through the Artifact plane and that users see
only governed playback, catalog, or bounded projection surfaces. Measure cache, spool,
projection scratch, and free-space headroom before activation. When the release declares
a hard-cut catalog migration, decide retention and export before activation; do not
invent compatibility with rows, manifests, or staged bytes from an older contract.

### Evidence Record

For every check, retain:

- date, environment, actor, and command or procedure;
- selected source, image, chart, and configuration identities;
- sanitized result and evidence location;
- explicit status: `not-started`, `in-progress`, `awaiting-approval`, `passed`,
  `failed`, `blocked`, `not-executed`, `not-applicable`, or `accepted-limit`;
- defect or decision owner;
- any limitation that prevents production reliance.

## Stage 8: Recovery And Clean Reproduction

### Ordinary Restart

Restart one declared ownership boundary at a time without deleting persistent storage.
Capture resource identities and sanitized business-state fingerprints before and after.
Verify identity, Gateway policy, business reads, audit, artifact or recording access,
and dependency health after recovery.

### Backup And Restore

Create a backup through the enterprise-owned procedure and restore it into an approved
target. Prove application-level readability, not merely the presence of restored bytes.
Record RPO, RTO, encryption, integrity, and the owners who may initiate restoration.

### Clean Reproduction

Recreate a disposable installation from the approved configuration revision and
immutable release identities. Do not rely on previous PVCs, temporary credential
material, node image caches, shell history, or undocumented patches. Run the complete affected
acceptance again and record every manual prerequisite in the installation repository or
runbook.

Clean reproduction and ordinary restart prove different properties. Destructive
reproduction requires explicit approval and an inventory of protected resources.

### Gate

- ordinary workload recovery preserves accepted state;
- backup restore produces readable business data;
- clean reproduction follows only documented inputs and ordering;
- known product defects and excluded recovery paths remain explicit;
- no protected enterprise resource is deleted as a convenience.

## Stage 9: Handoff

Before production use, hand over:

- architecture and ownership record;
- installation repository and release evidence;
- non-secret configuration and Secret inventory;
- identity and authorization model;
- monitoring dashboards and alert routes;
- backup and restore procedures with recent evidence;
- upgrade, rollback, maintenance, and certificate procedures;
- incident response and escalation contacts;
- known defects, limitations, and deferred scope;
- administrator and operator training;
- acceptance sign-off and residual-risk acceptance.

Confirm that the enterprise can operate the installation without relying on an
undocumented action performed by the delivery team.

## Stage 10: Ongoing Operation

Operate changes through the installation's normal review and reconciliation process.
Track artifact currency, compatibility, vulnerabilities, capacity, certificates,
credentials, backups, restore tests, access reviews, and domain acceptance.

An upgrade is a new reviewed release. Repeat affected technical and business acceptance
and preserve the previous known-good identities until rollback constraints are
resolved.

## Final Installation Gate

An installation is ready for the agreed use case when:

- ownership is explicit and the desired state has one reconciler;
- artifact and configuration identities are immutable and recorded;
- Secrets and identity are enterprise-owned and validated without disclosure;
- infrastructure, storage, networking, trust, and GPU prerequisites pass;
- selected services and gateway upstreams are healthy;
- human and machine authorization pass positive and negative checks;
- the bounded business workflow produces its expected observable result;
- audit and business evidence remain readable after ordinary restarts;
- ordinary restart, backup restore, and clean reproduction have recorded results;
- backup, restore, upgrade, rollback, monitoring, and support responsibilities are
  accepted;
- no required decision or action remains `awaiting-approval`;
- every `failed`, `blocked`, `not-executed`, or `accepted-limit` check is recorded and
  accepted by its owner.

Readiness applies to the agreed scope, environment, and release. It is not a permanent
claim that every Veoveo capability or future customer workflow is certified.
