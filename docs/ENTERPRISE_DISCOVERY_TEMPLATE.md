# Enterprise Discovery Template

Use this template to begin a Veoveo engagement with a new organization. It provides a
repeatable structure without assuming that every company has the same systems,
regulations, workflows, or deployment model.

Discovery is complete enough to begin solution design when the initial business
outcome, authority boundaries, system owners, installation constraints, and acceptance
owners are explicit. Unanswered questions may remain, but each must have an owner and a
date or event that will resolve it.

Do not collect passwords, private keys, access tokens, production datasets, or other
Secret values in this document. Record the owning system and approved delivery
mechanism instead.

## Document Control

| Field | Entry |
|---|---|
| Organization | |
| Engagement name | |
| Discovery owner | |
| Enterprise sponsor | |
| Technical owner | |
| Security owner | |
| Operations owner | |
| Version | |
| Last updated | |
| Classification | |
| Decision repository | |

## Executive Context

### Organization

- What does the organization do?
- Which business unit or operating area is involved?
- Which locations, regions, subsidiaries, or legal entities are in scope?
- What prompted the engagement now?
- Which existing initiative, contract, or deadline affects the work?

### Desired Outcome

Describe the outcome in business terms. Avoid starting with a product feature or a
specific technology.

| Question | Answer |
|---|---|
| What should become faster, safer, more reliable, or newly possible? | |
| Who benefits from the outcome? | |
| What happens today? | |
| What measurable change would justify the project? | |
| What is the cost or risk of doing nothing? | |
| What must not change? | |

### Initial Scope

| In scope | Out of scope | Possible later phase |
|---|---|---|
| | | |

State the smallest useful first workflow. Prefer one observable business result over a
broad commitment to connect every system.

## People And Decisions

### Stakeholders

| Person or team | Responsibility | Decision authority | Availability |
|---|---|---|---|
| | | | |

Include business users, application owners, platform engineering, identity, security,
networking, data governance, legal or compliance, operations, and support where
applicable.

### Users And Roles

| User group | Work performed | Information needed | Actions needed | Approval boundary |
|---|---|---|---|---|
| | | | | |

Questions to resolve:

- Which users are employees, contractors, partners, or service identities?
- Which roles may read, propose, approve, execute, administer, or audit?
- Which duties must remain separated?
- Which actions require two-person approval or an external change process?
- Which emergency access process exists, and who reviews its use?
- How are joiners, movers, and leavers handled?

## Business Workflow

Document the current and intended workflow before choosing integrations.

### Current Workflow

| Step | Actor | Input | System | Decision | Output | Current problem |
|---|---|---|---|---|---|---|
| | | | | | | |

### Intended Workflow

| Step | Actor | Veoveo capability | Human decision | External effect | Required evidence |
|---|---|---|---|---|---|
| | | | | | |

For each proposed automated action, answer:

- Is the action read-only, advisory, reversible, or irreversible?
- What is the maximum permitted scope of one action?
- What conditions must be checked before execution?
- Which result proves that the action occurred?
- What happens when execution is ambiguous or partially complete?
- Can the operation be made idempotent?
- Who owns recovery or compensation?

### External Effects And Replay

| Operation | Business identifier | Technical identifier | Durable reservation | Exact effect observation | Replay rule | Recovery owner |
|---|---|---|---|---|---|---|
| | | | | | | |

Clarify where an execution becomes durable relative to an external effect. Define what
happens when the caller loses the response after the external system has acted. A retry
must not repeat an irreversible operation merely because the business record is still
incomplete.

Record identical replay, conflicting replay, operator recovery, and compensation as
separate cases. Do not assume that every external system supports idempotency or
rollback.

## Systems And Integrations

### System Inventory

| System | Business owner | Technical owner | Purpose | Environment | Interface | Data classification | Change constraints |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

For each candidate system, capture:

- supported APIs, events, files, databases, or existing MCP service;
- authentication method and service-identity owner;
- network location and connectivity restrictions;
- rate, concurrency, payload, and availability limits;
- test environment and representative non-production data;
- vendor support and change windows;
- expected read operations and permitted write operations;
- authoritative source and conflict-resolution rules.

### Interface Semantics

| Interface | Version | Identifier rules | Pagination or truncation | Maximum response | Unknown fields | Error model | Owner |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

Ask whether reported counts describe a complete population or only the returned page.
Record numeric constraints, ordering, finite-value requirements, schema evolution, and
how real payload sizes will be measured. Business identifiers and external technical
identifiers need separate types and an explicit mapping owner.

### Data Movement

| Data set or stream | Source | Consumer | Direction | Volume and rate | Retention | Residency | Sensitivity |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

Clarify whether data may be copied, cached, indexed, recorded, transformed, exported,
or used by an AI model. Identify fields that require masking, minimization, or complete
exclusion.

### AI And Model Boundaries

- Which model providers are approved?
- Must inference stay inside a private network or jurisdiction?
- Which information may enter model context?
- Which outputs require human review?
- Are prompts, responses, evaluations, or model inputs retained?
- Which accuracy, latency, explainability, or cost constraints matter?
- Which decisions must never be delegated to a model?

## Identity And Authorization

| Topic | Current state | Required state | Owner |
|---|---|---|---|
| Identity provider and protocol | | | |
| User and group source | | | |
| Role and claim model | | | |
| Service identities | | | |
| MFA and conditional access | | | |
| Session duration | | | |
| Privileged administration | | | |
| Access review | | | |

Record the canonical public origin, OAuth audiences, redirect URI ownership, issuer,
tenant model, and logout expectations during solution design. Do not invent these
values during deployment.

### Endpoint And Authority Inputs

| Purpose | Public origin | Transport location | HTTP authority | OAuth resource or audience | TLS name | Proxy owner |
|---|---|---|---|---|---|---|
| | | | | | | |

Determine how ingress and proxies preserve scheme, host authority, redirect URIs, and
secure cookies. Record separate browser and machine flows. A successful administrator
login does not prove an operator flow or a service assertion.

## Security, Privacy, And Compliance

| Requirement | Source | Applies to | Evidence required | Owner |
|---|---|---|---|---|
| | | | | |

Discuss:

- information classification and handling rules;
- applicable laws, contracts, standards, and internal policies;
- encryption and key-management ownership;
- vulnerability, image, dependency, and configuration scanning;
- audit-event retention and access;
- incident notification and investigation;
- supplier and subprocessors review;
- penetration testing or independent assurance;
- data-subject, legal-hold, deletion, and export obligations;
- prohibited data, actions, destinations, and operating modes.

An engagement must not claim regulatory compliance merely because the platform exposes
security controls. The enterprise decides the applicable control framework and accepts
the complete operating environment.

## Infrastructure And Hosting

### Environments

| Environment | Purpose | Data allowed | Users | Reconciliation owner | Availability target |
|---|---|---|---|---|---|
| Development | | | | | |
| Test or acceptance | | | | | |
| Production | | | | | |
| Disaster recovery | | | | | |

### Platform Inputs

| Area | Decision or constraint | Owner |
|---|---|---|
| Kubernetes provider and version | | |
| Node architecture and capacity | | |
| NVIDIA GPU models and allocation | | |
| Private OCI registry | | |
| Git and GitOps controller | | |
| Ingress or private access | | |
| DNS and certificates | | |
| Network policies and egress | | |
| Persistent storage classes | | |
| Object storage | | |
| Secret manager and projection | | |
| Monitoring and logging | | |
| Backup infrastructure | | |

### Runtime Artifact Closure

| Image or artifact class | Approved source | Mirror required | Node pull path | Offline requirement | Owner |
|---|---|---|---|---|---|
| Platform images | | | | | |
| Extension images | | | | | |
| Kubernetes sandbox image | | | | | |
| Init and administrative Jobs | | | | | |
| GPU and device components | | | | | |
| Databases and object storage | | | | | |

Test DNS and registry access from cluster nodes and representative Pods. Publisher-host
access and its image cache do not prove node-runtime access.

### Storage Pressure And Growth

| Filesystem or store | Current capacity | Expected growth | Warning threshold | Expansion owner | Cleanup authority |
|---|---|---|---|---|---|
| Node filesystem | | | | | |
| Image filesystem | | | | | |
| Registry | | | | | |
| Database | | | | | |
| Object storage | | | | | |
| Recordings | | | | | |

Capture kubelet eviction thresholds, image garbage-collection behavior, inode limits,
and the free-space margin needed to pull and unpack an upgrade. Define which caches may
be pruned and which volumes, images, or persistent data are protected.

Determine whether the environment is connected, restricted-egress, or offline. Record
proxy, private certificate authority, firewall, registry mirror, and artifact-transfer
requirements.

## Reliability And Operations

| Topic | Requirement | Owner | Evidence |
|---|---|---|---|
| Service hours | | | |
| Availability objective | | | |
| Recovery time objective | | | |
| Recovery point objective | | | |
| Backup frequency and retention | | | |
| Capacity and growth | | | |
| Monitoring and alert routing | | | |
| Incident response | | | |
| Maintenance windows | | | |
| Upgrade cadence | | | |
| Rollback authority | | | |
| Support escalation | | | |

Identify the enterprise team that will operate each dependency. A working installation
cannot compensate for an unowned identity provider, registry, database, certificate,
or backup process.

## Installation Ownership

Use this table to prevent two systems or teams from concurrently managing the same
resource.

| Concern | Enterprise owner | Delivery mechanism | Durable source | Approval required |
|---|---|---|---|---|
| Cluster prerequisites | | | | |
| Veoveo release selection | | | | |
| Installation configuration | | | | |
| Secrets and private keys | | | | |
| Identity configuration | | | | |
| Gateway policy and bindings | | | | |
| Private extensions | | | | |
| DNS, TLS, and ingress | | | | |
| Storage and backups | | | | |
| Reconciliation | | | | |
| Acceptance evidence | | | | |

## Acceptance Definition

Define technical and business acceptance independently.

### Technical Acceptance

| Check | Method | Expected result | Evidence owner |
|---|---|---|---|
| Artifact identity | | | |
| Desired-state revision | | | |
| Workload readiness | | | |
| Storage attachment | | | |
| Identity login | | | |
| Authorization boundaries | | | |
| MCP discovery and health | | | |
| GPU execution where required | | | |
| Backup and restore | | | |
| Upgrade and rollback | | | |

### Business Acceptance

| Scenario | Actor | Starting state | Action | Observable result | Evidence retained |
|---|---|---|---|---|---|
| | | | | | |

Include negative cases: unauthorized user, invalid input, partial dependency failure,
duplicate request, unavailable provider, and restart during or after work.

### Catalog And Exposure Acceptance

| Identity | Expected tools | Expected resources | Explicitly hidden capabilities | Expected denial |
|---|---|---|---|---|
| | | | | |

Validate discovery and direct invocation separately. Human users must not inherit the
internal tools used by an orchestrating service. Machine identities require their own
minimum capability profiles.

When a service supplies an interactive App, record its exact `ui://` resource, linked
tools, readable resources, and any cross-server resource dependency. Decide whether the
App needs reactive updates, durable tasks, internal navigation, or agent messaging.
Each dependency must name the smallest required resource family and must remain subject
to the caller's Gateway policy; App discovery is not an independent authorization path.

### Recording Requirements

Complete this section only when the workflow uses recordings.

| Question | Decision |
|---|---|
| Which producer, tenant-local dataset key, application, and recording key identify the source? | |
| What are the durable dataset UUID and recording UUID, and who creates each? | |
| Which capture, properties, derived, and Blueprint layers are required? | |
| How is the recording correlated durably to a business execution? | |
| Must attachment occur before an external action? | |
| What proves Live availability? | |
| Who finalizes the producer stream? | |
| What operation publishes the selected release's sealed playback manifest? | |
| Which catalog revision and admitted recording set must the access grant bind? | |
| Does an App require a bounded Arrow projection rather than complete RRD access? | |
| What cache, projection scratch, spool, and free-space budgets are required? | |
| What retention, export, legal hold, and deletion rules apply? | |
| Which roles may read, seal, administer, or export recordings? | |

Treat discovery, Live, finalization, sealing, manifest publication, and History as
different lifecycle states. Timestamps alone do not prove that a recording belongs to
a business execution. The producer recording key is source metadata, not the durable
dataset or recording identity.

## Delivery Risks And Assumptions

### Risks

| Risk | Probability | Impact | Mitigation | Owner | Trigger |
|---|---|---|---|---|---|
| | | | | | |

### Assumptions

| Assumption | How it will be validated | Owner | Due |
|---|---|---|---|
| | | | |

### Dependencies

| Dependency | Needed by | Provider | Commitment | Status |
|---|---|---|---|---|
| | | | | |

### Known Product Limitations

| Limitation or defect | Affected capability | First workflow impact | Product owner | Enterprise decision | Retest trigger |
|---|---|---|---|---|---|
| | | | | | |

An enterprise may accept a bounded product limitation when the agreed workflow remains
valid. The acceptance must name unavailable behavior and residual risk. Installation
work must not silently patch core product state or broaden authority to hide the defect.

## Open Questions And Decisions

### Open Questions

| Question | Why it matters | Owner | Due | Status |
|---|---|---|---|---|
| | | | | |

### Decision Log

| Date | Decision | Alternatives | Reason | Approver | Revisit trigger |
|---|---|---|---|---|---|
| | | | | | |

## Discovery Exit Review

Discovery may move into solution design when:

- the first business workflow and measurable outcome are explicit;
- in-scope and out-of-scope boundaries are agreed;
- business, technical, security, and operational owners are named;
- source systems and authoritative data are identified;
- human and machine authority boundaries are understood;
- hosting, identity, network, Secret, and storage owners are identified;
- the enterprise has described applicable security and compliance obligations;
- technical and business acceptance owners agree on observable outcomes;
- unresolved questions have owners and resolution dates;
- no Secret values or sensitive production data are embedded in discovery records.

The output is an approved discovery record and a bounded solution-design backlog. It is
not yet an installation configuration or a promise that every requested integration is
supported.
