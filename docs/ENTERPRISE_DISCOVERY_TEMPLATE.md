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

