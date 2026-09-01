# Enterprise Installation Readiness

This document is the installation agent's durable control record for a fielded Veoveo
installation. It converts the guided customer conversation, authorized environment
discovery, and solution design into decisions, prerequisites, approvals, acceptance
evidence, operating ownership, and explicit residual risk.

The [enterprise installation runbook](ENTERPRISE_INSTALLATION_RUNBOOK.md) governs the
agent's behavior. [Enterprise discovery](ENTERPRISE_DISCOVERY_TEMPLATE.md) supplies its
adaptive questions. This record preserves what the organization decided, what the agent
observed or executed, what evidence exists, and what must happen next. The customer does
not need to populate it manually.

Record only values approved for this installation. Do not store passwords, tokens,
private keys, production payloads, or other Secret values here.

## Standards And Protocols

The record may cover Kubernetes, Helm, OCI Distribution, GitOps, OpenID Connect,
OAuth 2.0, TLS, DNS, MCP Streamable HTTP, JSON Schema, enterprise Secret projection,
object storage, persistent volumes, and Veoveo's versioned deployment and extension
contracts. Record the exact version and supported profile selected by the installation.
An internal adapter or acceptance harness is not a public enterprise contract.

## How To Use This Record

The installation agent creates one copy per installation environment as soon as the
organization, environment, and installation owner are identifiable. Link shared
decisions instead of silently assuming that development, acceptance, production, and
disaster-recovery environments are identical.

Update the record after every owner decision, approved inspection, plan approval,
execution step, acceptance check, failure classification, and recovery action. Each
entry distinguishes:

- an enterprise decision from an observed environmental fact;
- a proposed action from an approved or executed action;
- expected evidence from evidence actually collected;
- a product contract from an installation-specific choice;
- a blocker from a failed check;
- an accepted limitation from a passed capability.

Chat summaries and terminal history may help during one session, but they are not the
durable record. Store sanitized evidence in the enterprise-approved destination and link
it here. Never copy Secret values, private material, sensitive payloads, or unredacted
identity tokens into this document.

Every check that reports progress uses one status:

| Status | Meaning |
|---|---|
| `not-started` | No owner-approved work or evidence exists |
| `in-progress` | Work has started, but the gate has not passed |
| `awaiting-approval` | A bounded plan exists and the named owner must approve it before execution |
| `passed` | The stated check passed for the recorded release and environment |
| `failed` | The check ran and produced a product or composition failure |
| `blocked` | A named prerequisite or external decision prevents execution |
| `not-executed` | The check was required for this release but did not run |
| `not-applicable` | The owner approved that the requirement does not apply |
| `accepted-limit` | The owner accepted a bounded limitation for the agreed use case |

Never turn `failed`, `blocked`, or `not-executed` into `passed` through explanatory
prose. A platform health endpoint does not prove that its workloads, identity flow,
business workflow, persistence, or recovery passed.

## Conversation And Execution State

The agent keeps this section current so another authorized agent or owner can resume
without relying on the previous chat.

| Field | Recorded value |
|---|---|
| Current runbook stage | |
| Agent operating mode | `discover`, `plan`, `execute`, or `recover` |
| Current gate | |
| Last confirmed customer decision | |
| Last observed fact and evidence | |
| Approved plan identifier and scope | |
| Approval owner and timestamp | |
| Last executed step and result | |
| First causal failure, if any | |
| Protected resources or state | |
| Open question or blocker | |
| Next question, approval, or bounded action | |

Before executing a mutation, this state must identify the approved scope, environment,
expected effect, evidence, recovery boundary, and approver. After execution, replace the
planned result with the sanitized observed result. Do not carry approval from one stage,
environment, or materially different plan into another.

## Installation Identity

| Field | Recorded value | Evidence |
|---|---|---|
| Organization | | |
| Installation and environment | | |
| Business workflow | | |
| Installation owner | | |
| Business acceptance owner | | |
| Security acceptance owner | | |
| Operations owner | | |
| Configuration repository and revision | | |
| Veoveo compatibility release | | |
| Canonical public origin | | |
| Change window | | |
| Evidence repository | | |
| Final decision date | | |

## Scope And Authority

| Requirement | Decision or expected result | Owner | Evidence | Status |
|---|---|---|---|---|
| First bounded workflow | One observable business result is named | | | |
| In-scope capabilities | Selected reads, proposals, approvals, mutations, artifacts, and recordings are listed | | | |
| Explicit non-goals | Deferred integrations and product capabilities are listed | | | |
| Human roles | Read, propose, approve, execute, administer, and audit are separated as required | | | |
| Machine identities | Each service identity has one purpose and minimum authority | | | |
| Tenant boundary | Tenant ownership and isolation are explicit | | | |
| Work Context boundary | People, agents, services, data, and policy membership are explicit | | | |
| Emergency authority | Break-glass ownership, expiry, and review are defined | | | |
| Prohibited behavior | Forbidden data, destinations, actions, and automation modes are recorded | | | |
| Core-product boundary | Product defects are recorded without silently expanding installation scope | | | |

## Ownership And Reconciliation

| Resource or concern | Authoritative owner | Durable source | Reconciler | Change approver | Status |
|---|---|---|---|---|---|
| Cluster and node configuration | | | | | |
| GPU drivers and allocation | | | | | |
| Registry and mirrors | | | | | |
| Identity provider | | | | | |
| Secret manager and projection | | | | | |
| DNS, ingress, and certificates | | | | | |
| Platform Helm release | | | | | |
| Gateway composition | | | | | |
| Each external extension | | | | | |
| Databases and persistent storage | | | | | |
| Object storage | | | | | |
| Backups and restore | | | | | |
| Monitoring and incident response | | | | | |
| Acceptance evidence | | | | | |

Each Kubernetes object and external registration has one reconciler. A direct Helm
operator and a GitOps controller must not manage the same release concurrently.

## Infrastructure Readiness

| Requirement | Expected proof | Owner | Evidence | Status |
|---|---|---|---|---|
| Supported Kubernetes version | Version and support policy match the selected release | | | |
| Node architecture | Every selected image supports the node architecture | | | |
| Hardware GPU | Required NVIDIA devices, drivers, runtime, allocation, and in-workload execution pass | | | |
| CPU and memory | Requests, limits, scheduling margin, and growth assumptions are approved | | | |
| Node filesystem | Capacity and inode thresholds leave documented operating and upgrade margin | | | |
| Image filesystem | Image pull, unpack, garbage collection, and eviction thresholds are measured | | | |
| Persistent storage | Storage classes, access modes, expansion, performance, and failure domains are approved | | | |
| Object storage | Endpoint, authentication, encryption, lifecycle, capacity, and recovery pass | | | |
| Cluster DNS | Internal service resolution and approved external resolution pass from workloads | | | |
| Time synchronization | Nodes and identity-sensitive services use an approved time source | | | |
| Network policy support | The selected CNI enforces required ingress and egress policy | | | |
| Load balancer or ingress | Address allocation, health, source preservation, and failover are proven | | | |

Record both control-plane readiness and workload readiness. Kubernetes `/readyz=ok`
does not prove that application pods are scheduled or Ready. Disk pressure, image pull,
taints, PVC attachment, and dependency failure remain separate checks.

### Capacity Baseline

| Resource | Current use | Expected peak | Warning threshold | Expansion trigger | Owner |
|---|---|---|---|---|---|
| Node filesystem bytes and inodes | | | | | |
| Image and build cache | | | | | |
| Registry storage | | | | | |
| Database storage | | | | | |
| Object storage | | | | | |
| Recording ingest and archive | | | | | |
| Network ingress and egress | | | | | |
| GPU memory and utilization | | | | | |

## Artifact And Supply-Chain Closure

| Artifact | Source revision | Published identity | Platform | Mirror or transfer | Verification | Status |
|---|---|---|---|---|---|---|
| Platform chart | | | | | | |
| Platform images | | | | | | |
| Extension charts | | | | | | |
| Extension images | | | | | | |
| Gateway composition | | | | | | |
| Deployment lock or equivalent | | | | | | |
| Offline bundle, if used | | | | | | |

Inventory every transitive runtime image, not only Veoveo images. Init containers,
Jobs, sandbox images, GPU components, database images, and administrative utilities
must be pullable from the approved path. A connected publisher host does not prove that a
cluster node can resolve or pull from the same registry.

For a restricted or offline installation, prove exact image references in the runtime's
actual image store and namespace. A similarly tagged image in a publisher-host cache is
not sufficient evidence.

## Addressing, Network, And Trust

### Endpoint Authority Register

| Purpose | Transport URL | HTTP authority | Public resource or audience | TLS name | Network path | Owner |
|---|---|---|---|---|---|---|
| Public Console | | | | | | |
| Gateway OAuth authorization | | | | | | |
| Gateway OAuth token | | | | | | |
| Gateway MCP profile | | | | | | |
| Identity issuer | | | | | | |
| Each internal upstream | | | | | | |
| Object storage | | | | | | |

Transport location, HTTP `Host` authority, OAuth resource or audience, issuer, redirect
URI, and TLS server name are distinct values. Record them separately. Proxies and
port-forwards must preserve the authority expected by the selected route.

| Check | Expected proof | Owner | Evidence | Status |
|---|---|---|---|---|
| Canonical origin | One owner-approved origin drives browser-facing metadata | | | |
| Private DNS | Every service name resolves from its actual caller | | | |
| External DNS | Required registries, identity endpoints, and providers resolve from nodes and pods | | | |
| TLS chain | Server name, trust roots, expiry, and rotation pass | | | |
| Proxy behavior | Forwarded scheme, authority, callback, and secure-cookie behavior are correct | | | |
| Ingress policy | Only declared callers and ports enter each workload | | | |
| Egress policy | DNS, identity, Gateway, storage, and domain dependencies are explicitly allowed | | | |
| Denied path | A caller outside policy is rejected | | | |

## Identity And Secret Closure

### Human Identity

| Check | Expected result | Evidence | Status |
|---|---|---|---|
| Discovery and issuer | Exact canonical issuer and endpoints | | |
| Authorization flow | Approved Authorization Code and PKCE profile completes | | |
| Redirect URI | Exact registered callback is used | | |
| Secure cookies and proxy | Browser and proxy preserve the required cookie attributes | | |
| Claims | Subject, tenant, roles, groups, scopes, and clearance are typed and correct | | |
| Session and logout | Duration, renewal, revocation, and logout match policy | | |
| Joiner/mover/leaver | Access changes propagate within the approved interval | | |
| Administrator separation | Administrative authority is not inherited by ordinary users | | |

### Machine Identity And Key Association

| Principal | Authentication method | Issuer and subject | Audience | Public key or trust source | Private Secret contract | Allowed capabilities | Status |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

For every signing identity, prove mathematically that the mounted private key matches
the registered public key. Record key identifiers, algorithms, intended use, file modes,
rotation owner, and overlap procedure without recording private material. A valid key
paired with the wrong JWKS is an authentication failure, not a reason to broaden trust.

| Secret contract | Required keys | Authority | Destination | Rotation and restart | Non-disclosure proof | Status |
|---|---|---|---|---|---|---|
| | | | | | | |

Provision Secret contracts before dependent workloads. Never alternate temporary
material and installation material as an in-place repair.

## Gateway, MCP, And Extension Closure

| Requirement | Expected proof | Owner | Evidence | Status |
|---|---|---|---|---|
| Upstream declaration | Canonical MCP URL and health URL are present | | | |
| Service discovery | Service name, namespace, port, and labels match the release | | | |
| Probe authority | Health probe host and allowed-host policy agree | | | |
| Protocol profile | Negotiated MCP version and supported surface are recorded | | | |
| Protocol metadata | Required version metadata and transport headers reach every request | | | |
| Response bounds | Payload limits reflect measured real responses and parsing behavior | | | |
| Typed models | Complete responses, unknown fields, finite numbers, and truncated collections are handled intentionally | | | |
| Extension packaging | SDK and source layout required by the build context are present | | | |
| Runtime user | Non-root identity, filesystem, capabilities, and mounts pass | | | |
| Network policy | Required Gateway, identity, DNS, and domain egress pass without broad access | | | |
| Conformance | Hosted-server conformance passes for the published image | | | |
| App discovery | Caller-visible `ui://` resources, titles, icons, and linked tools match the approved catalog | | | |
| App isolation | Sandbox, CSP, cookies, storage, and bridge permissions match the selected App contract | | | |
| Reactive catalog | Partial discovery and resource notifications update without blocking unrelated Apps | | | |
| Cross-server resources | Every dependency is installation-declared, prefix-bounded, policy-filtered, and read through Gateway | | | |

### Capability Exposure Matrix

Record exact catalogs. A successful call does not prove that discovery is correct, and
an absent catalog entry does not prove that a direct call is denied.

| Identity | Expected tools | Expected resources | Explicitly absent | Representative allowed call | Representative denied call | Status |
|---|---|---|---|---|---|---|
| | | | | | | |

Human profiles must not receive internal domain tools merely because an internal
adapter requires them. Give machine identities the smallest dedicated profile needed
for their declared operation.

## Business Workflow And External Effects

### Workflow Contract

| Step | Actor | Input type | Preconditions | Authorized operation | Observable result | Durable evidence |
|---|---|---|---|---|---|---|
| | | | | | | |

Keep business identifiers separate from technical identifiers supplied by external
systems. Record mapping ownership and validation. Never force an external identifier
into a narrower enterprise identifier type merely because both are strings.

For each external response, decide whether list counts describe the full population or
the returned collection. Record truncation, pagination, response-size limits, unknown
field behavior, and numeric validity. Calibrate limits against real runtime payloads.

### Distributed Idempotency And Ambiguous Failure

| Milestone | Durable state | Written before or after external effect | Replay behavior | Recovery owner |
|---|---|---|---|---|
| Request admitted | | | | |
| Execution reserved | | | | |
| External effect confirmed | | | | |
| Business evidence persisted | | | | |
| Audit completed | | | | |

When a workflow can mutate an external system, reserve the execution durably before the
effect. A retry with the same idempotency identity reuses the reservation. Conflicting
payloads fail closed. Do not infer success, repeat an irreversible action, or invent a
rollback after an ambiguous failure.

The acceptance procedure must retain its replay identity long enough to execute the
approved replay checks without printing or committing it. Losing the input does not
authorize generating a different identity and calling that an identical replay.

### Workflow Acceptance

| Scenario | Initial state | Action | Immediate exact observation | Final state | Audit correlation | Status |
|---|---|---|---|---|---|---|
| Positive authorized flow | | | | | | |
| Unauthorized role | | | | | | |
| Invalid input | | | | | | |
| Identical replay | | | | | | |
| Conflicting replay | | | | | | |
| Dependency unavailable before effect | | | | | | |
| Failure after effect | | | | | | |

Use an exact domain observation. A global metric or probabilistic change does not prove
that the selected operation produced the intended effect.

## Persistence, Data, And Recording

| State | Authoritative store | Retention | Backup | Restore proof | Restart proof | Owner | Status |
|---|---|---|---|---|---|---|---|
| Identity | | | | | | | |
| Platform control data | | | | | | | |
| Extension business state | | | | | | | |
| Artifacts | | | | | | | |
| Recordings and journals | | | | | | | |
| Audit | | | | | | | |

### Recording Correlation And Lifecycle

| Requirement | Expected proof | Evidence | Status |
|---|---|---|---|
| Governed discovery | Recording is found through an authorized catalog | | |
| Exact selection | Dataset UUID/key, recording UUID, application, producer key, tenant, and Work Context match | | |
| Layer identity | Every admitted immutable layer has type, ordinal or revision, Artifact occurrence, length, digest, and schema identity | | |
| Catalog binding | Grant binds the catalog revision, admitted recording set, policy revision, actor, and expiry | | |
| Blueprint | Producer or dataset-default Blueprint is selected and served through its governed occurrence | | |
| Correlation | A durable attachment links business plan, execution, and recording ID | | |
| Attachment ordering | Reservation exists before the correlated external actuation | | |
| Live | One complete governed live message is consumed | | |
| Finalization | Producer closes normally and segments reach the documented terminal state | | |
| History | The selected release's sealed playback manifest is available through the governed route | | |
| Projection | Bounded projection enforces selectors, rows, bytes, deadline, digest, and one-time redemption when used | | |
| Capacity | Hub spool, MCP cache, projection scratch, and free-space floors satisfy the measured workload | | |
| Storage isolation | Browser receives no PVC, object-store path, or storage credential | | |
| Replay | Identical replay reuses the attachment without duplicate effect | | |

Temporal proximity does not prove correlation. Attach a recording to the durable
execution during the workflow. Keep recording evidence separate from actuation evidence
when their lifecycles differ. `live`, finalized data, sealing, sealed manifest, and
History availability are separate states and require separate proof. A producer key is
metadata and must not be substituted for the durable dataset or recording UUID. Record
the exact manifest schema selected by the release; do not assume compatibility across a
declared hard cut.

## Restart, Upgrade, Rollback, And Reproduction

### Restart Matrix

| Workload | Restart method | State that must survive | Dependency order | Readiness result | Business read result | Status |
|---|---|---|---|---|---|---|
| | | | | | | |

Restart one ownership boundary at a time without deleting persistent storage. Preserve
resource identities and compare sanitized business-state fingerprints before and after.
Transient readiness failures during normal startup are not incidents when the workload
recovers within its declared budget and does not restart.

### Upgrade And Rollback

| Scenario | From | To | Data migration | Rollback boundary | Evidence | Status |
|---|---|---|---|---|---|---|
| Identical reconciliation | | | | | | |
| Supported upgrade | | | | | | |
| Persistent schema transition | | | | | | |
| Failed release recovery | | | | | | |
| Rollback | | | | | | |

A failed Helm revision can coexist with Ready old pods. Record release status, desired
replicas, effective image IDs, and current ReplicaSet ownership instead of relying on
one signal.

Before a hard-cut data contract reaches a persistent store, inventory existing records.
Choose an explicit migration, a separately readable historical archive, or an approved
clean replacement. Never populate required fields with synthetic values merely to make
old state validate.

### Clean Reproduction

| Check | Expected result | Evidence | Status |
|---|---|---|---|
| Destructive boundary approved | Disposable and protected resources are enumerated | | |
| External services preserved | Shared registry, repositories, and unrelated resources remain intact | | |
| Empty environment | No previous installation state supplies an undocumented dependency | | |
| Ordered bootstrap | Infrastructure, Secrets, identity, platform, extensions, and Gateway follow the documented order | | |
| Immutable release | Exact source, chart, image digests, and lock are reused | | |
| Full acceptance | Identity, policy, workflow, recording, persistence, and negative tests repeat | | |
| Manual-step audit | Every required action exists in the runbook | | |

Clean reproduction is distinct from restarting an existing installation. A recovery
that depends on old caches, PVCs, credentials, or shell history is not clean evidence.

## Operations And Support

| Requirement | Owner | Procedure or destination | Evidence | Status |
|---|---|---|---|---|
| Metrics and dashboards | | | | |
| Alert routing | | | | |
| Log retention and redaction | | | | |
| Audit export and access review | | | | |
| Capacity review | | | | |
| Certificate and credential rotation | | | | |
| Backup monitoring and restore exercise | | | | |
| Vulnerability and artifact review | | | | |
| Incident response | | | | |
| Vendor and product escalation | | | | |
| Maintenance and upgrade cadence | | | | |

## Failure Classification

Classify the first causal failure before editing or retrying.

| Class | Definition | Default response |
|---|---|---|
| Installation configuration | An installation-owned value, binding, policy, Secret reference, route, or ordering is wrong | Correct the desired state and repeat affected acceptance |
| Infrastructure prerequisite | Cluster, node, GPU, DNS, registry, storage, port, or external dependency is unavailable | Restore the owned prerequisite and preserve application state |
| Enterprise extension | Customer or supplier extension behavior violates its declared contract | Correct that extension and republish immutably |
| Product defect | Supported Veoveo behavior fails independently of this composition | Record a focused product finding; do not modify core as installation work |
| Procedure or harness | The acceptance command, selector, authority, parser, or ephemeral harness is wrong | Correct the procedure and do not classify it as product behavior |
| Unrelated defect | The failure is outside the accepted workflow | Record it and continue only when it does not invalidate the gate |

### Defect And Limitation Register

| ID | Classification | Exact symptom | Affected gate | Workaround used | Owner | Decision | Retest trigger |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

An accepted product limitation remains visible. State which capability is unavailable,
which accepted workflow remains valid, and which owner accepted the residual risk. Do
not rewrite databases, journals, or persistent state to make an acceptance check green.

## Enterprise Input Qualification

Every operational input must resolve to an installation-owned decision before rollout.

| Input boundary | Enterprise decision required |
|---|---|
| Kubernetes environment | Supported platform, capacity, lifecycle, and ownership |
| Public and private endpoints | Canonical DNS, ingress, TLS, proxy, and callback paths |
| Identity system | Enterprise identity provider, clients, claims, and lifecycle |
| Credentials and signing material | Enterprise Secret authority, registration, rotation, and projection |
| OCI supply chain | Approved registry, mirrors, retention, scanning, and transfer |
| Direct Helm command | GitOps or approved direct-release ownership |
| Runtime image availability | Node-visible registry and complete transitive image closure |
| Namespaces and labels | Installation-owned naming, selectors, and policy boundaries |
| Human roles | Enterprise job roles and separation of duties |
| Work Contexts | Approved business context, membership, and authority |
| Persistent storage | Enterprise storage class, backup, restore, and lifecycle |
| Decommissioning | Approved retention, export, revocation, and deletion procedure |
| Acceptance credentials | Supported identity input owned by the acceptance procedure |
| Product limitation | Explicit residual risk or independently approved product fix |

No reference hostname, credential, identity, image cache, port, storage path, cleanup
command, or manual repair becomes an enterprise default.

## Acceptance Ledger

| ID | Layer | Check | Release and environment | Result | Evidence location | Owner | Limitation or follow-up |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

Record static validation, deployed runtime acceptance, browser or user acceptance,
negative authorization, persistence, restore, and clean reproduction separately. Tests,
Helm rendering, pod readiness, and business outcomes are complementary evidence.

## Control Rationale

This table records the operational reason for each readiness control without prescribing
a customer topology.

| Observed lesson | Enterprise control in this record |
|---|---|
| A baseline failure can predate the new integration | Record release, host, and existing-workflow baseline before mutation |
| Installation configuration and product behavior have different owners | Use the failure classification and defect register |
| Identity is installation-owned | Select the enterprise issuer, clients, claims, proxy, and lifecycle explicitly |
| Browser success depends on origin, callback, cookies, and proxy behavior | Record public origin, authority, redirect URI, TLS name, and secure-cookie path separately |
| A service may present a valid assertion signed by the wrong key | Prove every private-key and public-JWKS association |
| Generic and installation-specific Secrets can overwrite each other | Assign one Secret authority and enforce bootstrap ordering |
| A changed projected Secret may require workload reconciliation | Record rotation, overlap, restart, and rollback behavior |
| Helm success, Kubernetes API health, and workload readiness differ | Preserve layered infrastructure and release evidence |
| A failed upgrade may leave old pods serving | Inspect Helm revision, ReplicaSet ownership, effective image, and desired/Ready counts |
| Probe requests can use a different authority than ordinary traffic | Align probe host, allowed-host policy, service port, and route |
| A NetworkPolicy can admit the business service but block OAuth or Gateway access | Validate every required egress from the real caller namespace |
| Human policies can accidentally expose internal domain tools | Maintain exact positive and negative capability matrices per identity |
| A machine orchestrator needs narrower authority than a human profile | Use a dedicated service identity and capability profile |
| Internal transport URL and public OAuth audience are not interchangeable | Maintain the endpoint authority register |
| A federated MCP endpoint may expose prefixed tool names | Validate the composed catalog and call the canonical federated surface |
| Negotiated MCP metadata must reach every request, including library-generated ones | Test protocol metadata across the complete session lifecycle |
| Client transport cleanup can fail before a business request | Test initialize, list, read, failure cleanup, and close order |
| Real responses can contain more fields than an incomplete integration model | Validate complete typed responses and unknown-field policy |
| A total count can exceed a bounded returned collection | Record truncation semantics instead of asserting equality blindly |
| Response limits applied after parsing do not bound parser memory | Document where each limit applies and calibrate it with real payloads |
| A global metric may not prove one external change | Select an operation with an exact immediate domain observation |
| A durable business write after an external effect leaves an ambiguous retry window | Reserve execution before the effect and persist explicit milestones |
| A different idempotency identity after completion must not repeat the effect | Define identical replay and conflicting replay behavior |
| Historical data can violate a hard-cut schema | Decide migration, archive, or clean replacement before rollout; never invent evidence |
| Recording timestamps do not prove business correlation | Create a durable attachment to a specific execution during the workflow |
| Live, finalized segments, sealing, manifest publication, and History are distinct | Accept and evidence every recording lifecycle state separately |
| A recording may remain durable while a lifecycle worker cannot recover | Record the product limitation without rewriting journals or databases |
| Restarting one workload can reveal dependency order or stale connections | Use a workload restart matrix and exact post-restart business reads |
| A clean install can accidentally rely on old credentials or caches | Reproduce from immutable inputs with an empty-state and manual-step audit |
| A test harness may have fixed credentials or incorrect selectors | Classify procedure defects separately and never present them as product evidence |
| An operator host may resolve a registry while a cluster node cannot | Test node and Pod DNS and registry paths independently |
| An image can exist in Docker but not in kubelet's containerd store | Verify the effective runtime reference and image-store namespace |
| Sandbox, init, and administrative images can block the platform | Include every transitive image in supply-chain closure |
| `/readyz=ok` can coexist with unschedulable workloads | Inspect taints, DiskPressure, image pulls, PVCs, and pod conditions |
| Reclaimable cache can belong to an unselected BuildKit daemon | Identify the owning cache store before any approved cleanup |
| Free space above a hard eviction threshold may still miss the reclaim target | Record image-GC thresholds and upgrade operating margin |
| Recovery by one-time image import does not fix broken DNS | Separate controlled recovery from the durable infrastructure correction |
| Stateful acceptance depends on preserving PVC and object-store ownership | Record resource identities, backups, restores, and protected deletion boundaries |

If a future reference installation reveals a new class of failure, add the generalized
control here and update the owning discovery or runbook section in the same change.

## Final Decision

| Decision | Entry |
|---|---|
| Agreed production use case | |
| Passed gates | |
| Failed or blocked gates | |
| Accepted limitations | |
| Deferred scope | |
| Rollback position | |
| Business approval | |
| Security approval | |
| Operations approval | |
| Installation owner approval | |

The installation is ready only for the recorded release, environment, workflow, and
limitations. Approval does not certify every Veoveo capability or eliminate the need
for capacity review, access review, recovery exercises, and future release acceptance.
