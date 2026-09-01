# Veoveo, Explained

Veoveo helps an organization use its people, AI, data, and operational systems as one
governed environment. It connects what already exists, controls who may do what, and
keeps evidence of important results.

This guide explains the product without assuming knowledge of software development,
Kubernetes, or AI infrastructure.

<a href="images/overview-trusted-action.png">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="images/overview-trusted-action-dark.png">
    <img src="images/overview-trusted-action.png" alt="A business question moves through five stages: ask, understand approved information, approve who may act, perform the action, and prove the result with evidence">
  </picture>
</a>

*Veoveo turns information into governed action: observe, understand, decide, act, and
retain evidence.*

## In One Minute

Veoveo provides four things around an organization's existing systems:

- **A common view.** Approved information can be brought together instead of remaining
  scattered across separate applications.
- **Controlled action.** People and AI can use only the information and operations
  allowed for their identity and role.
- **Coordinated work.** Requests, approvals, long-running operations, and results remain
  visible through a shared operating environment.
- **Durable evidence.** Decisions, outputs, recordings, and audit events can be retained
  with the business activity that produced them.

> Veoveo does not replace the company's identity provider, source systems, or operating
> teams. It gives them a governed way to work together.

## A Day With Veoveo

Consider a transport company responding to congestion:

1. An operator asks what is happening in a particular area.
2. Veoveo confirms the operator's identity and business role.
3. Approved services bring together fleet, map, traffic, and incident information.
4. An application or AI agent proposes an adjustment.
5. Veoveo checks whether that person may approve or execute it.
6. The transport system performs the authorized action.
7. Veoveo retains the request, decision, result, and supporting evidence.

The same pattern applies to logistics, industrial operations, media analysis, planning,
field services, and simulation. The systems and rules change. The controlled path from
information to action remains the same.

## What People See

The Operations Console gives each signed-in user a view shaped by their authority. It
brings together active work, services, artifacts, agents, recordings, access decisions,
and platform health without exposing infrastructure credentials.

[![Veoveo Operations Console showing active work, services, artifacts, agents, recent decisions, and platform health](screenshots/gallery/console-overview.png)](screenshots/gallery/console-overview.png)

The Console is not the whole product. It is the visible workspace over a set of governed
services. Specialized applications can add maps, charts, recordings, video, or other
domain views while keeping the same identity and policy boundary.

## Five Ideas To Remember

| Idea | In everyday language |
|---|---|
| One enterprise installation | The organization has its own configured Veoveo environment and retains ownership of its infrastructure, credentials, data, and policies. |
| Every actor has an identity | A person, application, or technical service acts with known authority rather than broad anonymous access. |
| Work happens in context | People see the teams, information, services, and actions relevant to a defined business purpose. |
| Capabilities are explicit | Connected services declare what can be read or done. Access may be read-only, approval-based, or limited to one narrow operation. |
| Results keep their evidence | Work, audit events, files, and recordings preserve what happened and how the result was produced. |

AI agents fit inside this model. An agent can interpret a request, gather permitted
information, use approved capabilities, and produce a result. It does not receive
unrestricted access merely because it uses AI. A person can remain responsible for
approval whenever the business process requires it.

## Reactive And Proactive Work

Veoveo can support immediate response and forward planning at the same time.

<a href="images/operations-loop.png">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="images/operations-loop-dark.png">
    <img src="images/operations-loop.png" alt="Two continuing operating loops: reactive work detects, decides, and intervenes; proactive work analyzes, plans, and dispatches">
  </picture>
</a>

**Reactive work** starts with an event that needs attention: detect a problem, make a
decision, and intervene. **Proactive work** looks ahead: analyze conditions, prepare a
plan, and dispatch approved work. Both can use the same governed information and leave
evidence in the same operating environment.

## Trust Is Part Of The Path

For an important request, Veoveo separates four questions:

1. **Who is asking?** The enterprise identity provider authenticates the person or
   service.
2. **What may they do?** Policy considers role, business context, requested capability,
   and other approved conditions.
3. **Which system performs the work?** The connected domain service remains responsible
   for executing its business operation correctly.
4. **What proves the outcome?** Work state, audit events, artifacts, or recordings retain
   the evidence required by the workflow.

A denied request is also meaningful evidence. It shows that the boundary enforced the
organization's rules instead of silently passing the request through.

## What The Enterprise Decides

Each installation reflects decisions made with the organization:

- the first business outcomes and workflows to support;
- the people, roles, teams, and approval boundaries involved;
- the systems and information that may be connected;
- the actions that remain read-only or require human approval;
- the hosting, networking, domain, identity, storage, and recovery model;
- the security, privacy, retention, audit, and regulatory obligations;
- the owners for operation, support, incidents, and future upgrades;
- the evidence that will count as acceptance for each workflow.

Veoveo supplies product contracts and deployment artifacts. It does not silently make
these enterprise decisions.

## Shared Responsibility

<a href="images/overview-responsibility.png">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="images/overview-responsibility-dark.png">
    <img src="images/overview-responsibility.png" alt="The company defines its people, rules, and priorities; Veoveo connects, governs, coordinates, and records; business systems retain their data, applications, and operations">
  </picture>
</a>

| Veoveo provides | The enterprise owns | Connected systems own |
|---|---|---|
| The supported platform, governed service boundary, and release artifacts | Infrastructure, identity, networking, configuration, policies, data, and operation | Correct business behavior, availability, credentials, and effect evidence |
| Work, artifact, recording, policy, and audit capabilities selected for the installation | Acceptance, monitoring, backup, recovery, support, and residual risk | Stable interfaces, limits, and recovery behavior |

A healthy platform does not automatically prove that every external system or business
workflow is correct. Each layer is accepted on its own evidence.

## What Success Looks Like

A successful installation can answer these questions with evidence:

1. **Can it run?** The infrastructure can schedule workloads, reach dependencies, attach
   storage, and provide the required hardware.
2. **Can the right people use it?** Identity and policy expose exactly the intended
   information and actions, including expected denials.
3. **Does the business workflow work?** One approved real scenario produces an exact,
   observable result.
4. **Can the result be trusted later?** Accepted state, audit, artifacts, and recordings
   survive the agreed recovery procedures.
5. **Can the organization operate it?** Owners can monitor, support, upgrade, and recover
   the installation from documented inputs.

A green health indicator answers only part of the first question. Production readiness
requires the rest.

## A Practical Starting Point

An organization can begin with a simple request to an AI agent working with the Veoveo
repository:

> Help us install Veoveo. Guide us through the information and approvals you need.

The agent asks progressive questions, discovers technical facts when authorized,
prepares bounded plans, stops for owner decisions, and maintains the installation
evidence. The customer does not need to understand or complete every technical document
before the conversation starts.

An enterprise engagement normally moves through five stages:

**Discover → Define → Install → Prove → Operate**

Discovery identifies the desired outcome, decision owners, systems, constraints, and
risks. The team then defines one bounded workflow, installs only what it needs, proves
the result and authority boundaries, and transfers the accepted service into operation.

The first goal is not to connect everything. It is to establish one valuable workflow
that the organization can understand, verify, and operate.

## Terms You May Encounter

| Term | Plain-language meaning |
|---|---|
| Work Context | A governed business environment that groups the relevant people, services, information, and rules. |
| Agent | An AI-enabled worker operating within declared authority. |
| MCP service | A connected capability provider that offers approved information, actions, documentation, or an application. |
| App | A specialized interactive view, such as a map or recording viewer, governed by the same access boundary. |
| Artifact | A durable output such as a report, dataset, annotation, media file, or model. |
| Recording | Governed live or historical operational data that can preserve how a supported system changed over time. |

## Product Limits

Veoveo is not unrestricted AI access to company systems. It does not replace the
enterprise identity provider, network, backup program, security organization, or owners
of connected business systems.

Like any large platform, it can also have defects or capabilities that are unsuitable
for a particular workflow. An installation records the limitation, tests whether the
agreed outcome remains valid, and routes product defects to their owner. It does not
claim that an unavailable or untested capability passed.

## Continue From Here

The installation agent uses these sources behind the guided conversation:

- [Enterprise installation runbook](ENTERPRISE_INSTALLATION_RUNBOOK.md) governs the
  conversation, planning, approvals, execution, and handoff.
- [Enterprise discovery](ENTERPRISE_DISCOVERY_TEMPLATE.md) supplies adaptive questions.
- [Enterprise installation readiness](ENTERPRISE_INSTALLATION_READINESS.md) preserves
  decisions, evidence, limitations, blockers, and approvals.
- [Enterprise deployment](ENTERPRISE_DEPLOYMENT.md) defines the normative technical
  execution contract and procedure.
