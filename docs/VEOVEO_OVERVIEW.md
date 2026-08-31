# Veoveo Explained

Veoveo is a platform for connecting people, artificial intelligence, business data,
and operational systems in one governed environment. It helps an organization use AI
for real work without giving a model unrestricted access to every system.

This guide explains the product in business terms. It does not assume knowledge of
software development, Kubernetes, or AI infrastructure.

## The Basic Idea

Most organizations already have valuable systems: databases, document repositories,
planning tools, simulators, cameras, maps, business applications, and specialized
software. These systems often remain isolated. A person must move between them,
interpret their information, and coordinate each action manually.

Veoveo provides a governed operating layer across those systems. It presents approved
information and actions to people and AI agents through one controlled boundary. The
organization decides what is connected, who may see it, which actions are permitted,
and what evidence must be retained.

Veoveo does not replace every existing application. It connects selected capabilities
and makes them usable through consistent identity, authorization, audit, and operational
workflows.

## A Simple Example

Consider a transport company managing a city fleet.

1. An operator asks for the current traffic situation.
2. Veoveo identifies the operator and determines which business environment they may
   access.
3. Approved services provide fleet, traffic, map, and incident information.
4. An AI agent or application proposes a traffic adjustment.
5. Veoveo checks whether the operator may approve or execute that action.
6. The operational system performs the approved change.
7. Veoveo records what was requested, who authorized it, what happened, and which
   evidence belongs to the result.

The same structure can support logistics, industrial operations, media analysis,
planning, field services, simulation, or other domains. The connected systems and
business rules change, but the governed workflow remains recognizable.

## The Main Concepts

### Installation

An installation is the Veoveo environment operated for one organization. It contains
the services, integrations, identity configuration, storage, policies, and applications
selected for that organization.

An installation is not a shared public account controlled by Veoveo. The enterprise
decides where it runs and retains ownership of its infrastructure, credentials, data,
and operational policies.

### Identity And Access

Every person or technical service acts through an identity. Veoveo uses the
organization's approved identity system to determine who the actor is and what roles,
business environment, and permissions apply.

Access is not all-or-nothing. One person may read operational state, another may
approve a plan, and a dedicated service may execute one narrowly defined machine
operation. These permissions can be kept separate.

### Work Context

A Work Context is the governed business environment in which work occurs. It brings
together the relevant people, agents, services, data, and policies for a defined
purpose.

For example, a company might separate traffic operations, warehouse planning, and
executive reporting. A person's access in one context does not automatically grant
access in another.

### Agents

An agent is an AI-enabled worker operating within declared authority. It can interpret
requests, gather permitted information, use approved capabilities, and produce results.

An agent does not receive unrestricted system access. Its actions pass through the
same identity, policy, and audit boundaries as other actors. Some actions can require a
human decision before execution.

### MCP Services

MCP services are the governed capability providers connected to Veoveo. A service may
expose information to read, actions to perform, documentation, or a specialized user
interface.

Examples include maps, recordings, time-series analysis, optimization, simulations,
documents, and a company's private operational service. Veoveo can combine first-party
services with extensions owned by the enterprise or another supplier.

### Work

Work represents a request or operation handled by the platform. It has a lifecycle,
an owner, authorization decisions, results, and operational evidence. Long-running
work remains observable instead of disappearing into an untracked background process.

### Artifacts

Artifacts are durable outputs such as reports, datasets, annotations, media, models, or
other files produced or registered through governed work. Access rules remain attached
to the artifact rather than depending on an unprotected storage link.

### Recordings

Recordings preserve the evolving state of a supported system or simulation. They can
support live observation and, when the recording lifecycle is complete, historical
review. A recording is governed data: users receive an authorized view rather than
direct access to internal storage.

### Audit And Decisions

Veoveo records important authorization and operational decisions. The audit trail
answers questions such as:

- Who requested the action?
- Which policy allowed or denied it?
- Which service performed it?
- What result and evidence were produced?
- Which installation revision was active?

Audit evidence supports investigation and accountability. It does not replace the
organization's legal, compliance, or records-management program.

## How A Request Moves Through Veoveo

A typical request follows this path:

1. **Sign in.** The person authenticates through the organization's identity provider.
2. **Select governed context.** Veoveo establishes the tenant, Work Context, roles, and
   allowed applications.
3. **Discover capabilities.** The user or agent sees only the services and operations
   permitted for that identity.
4. **Read or propose.** The actor reads approved information or proposes an action.
5. **Authorize.** Veoveo evaluates policy before the request reaches the operational
   service.
6. **Execute.** The selected service performs the operation within its own contract.
7. **Preserve the result.** Work state, audit events, artifacts, or recordings retain
   the evidence required by the workflow.

This separation matters. Authentication proves identity, authorization decides what
that identity may do, and the domain service remains responsible for performing the
business operation correctly.

## What The Enterprise Chooses

Every installation is shaped by the enterprise. Decisions normally include:

- business outcomes and initial use cases;
- users, teams, roles, and approval boundaries;
- systems and data that may be connected;
- actions that remain read-only or require human approval;
- hosting location, networking, domain names, and certificates;
- identity provider and account lifecycle;
- storage, retention, backup, and recovery objectives;
- security, privacy, audit, and regulatory obligations;
- operational ownership, support, and incident response;
- acceptance criteria for each business workflow.

Veoveo supplies product contracts and deployment artifacts. It does not silently make
these enterprise decisions.

## What Veoveo Does Not Mean

Veoveo is not unrestricted AI access to company systems. It is not a replacement for
the enterprise identity provider, network, backup program, or security organization.
Installing the platform also does not prove that every business integration is ready.

Technical readiness and business acceptance are separate. A healthy service may still
need domain validation, user review, recovery testing, and operating procedures before
the organization relies on it.

## From First Conversation To Operation

A normal engagement progresses through several stages:

1. **Discovery:** understand the organization, desired outcomes, systems, constraints,
   and decision owners.
2. **Solution definition:** choose the first workflows, services, integrations, and
   authority boundaries.
3. **Installation design:** define infrastructure, identity, networking, storage,
   secrets, release ownership, and recovery.
4. **Implementation:** publish selected artifacts, configure the installation, and
   integrate enterprise-owned services.
5. **Acceptance:** prove identity, policy, service health, business outcomes, evidence,
   and recovery separately.
6. **Handoff and operation:** establish ownership, monitoring, support, upgrades,
   rollback, and periodic access review.

The first goal is not to connect everything. It is to establish one valuable,
well-bounded workflow that the organization can understand, verify, and operate.

## Further Reading

- [Enterprise discovery template](ENTERPRISE_DISCOVERY_TEMPLATE.md) guides the first
  conversations with a new organization.
- [Enterprise installation runbook](ENTERPRISE_INSTALLATION_RUNBOOK.md) organizes the
  technical delivery from approved scope through operational handoff.
- [Deployment guide](DEPLOYMENT_GUIDE.md) defines the detailed installation procedure.
- [Enterprise deployment](ENTERPRISE_DEPLOYMENT.md) defines ownership of artifacts,
  configuration, infrastructure, secrets, and reconciliation.

