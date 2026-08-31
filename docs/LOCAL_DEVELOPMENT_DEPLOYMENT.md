# Local development deployment

This is the operational entrypoint for running Veoveo on one Linux development
workstation. It assembles the existing local profile, k3d, Helm, Secret, image
publication, and acceptance procedures into one ordered workflow. It does not define a
new deployment contract.

The supported complete local profile is the SUMO showcase. It runs a real Kubernetes
cluster with the Veoveo platform, local Keycloak identity, the SUMO mobility simulator,
recording ingest, History playback, and Live playback.

This procedure is disposable development infrastructure. It is not evidence that an
enterprise or production installation is ready. Production ownership, identity,
secrets, release, upgrade, and rollback are documented in
[Enterprise deployment](ENTERPRISE_DEPLOYMENT.md).

Use the [enterprise installation runbook](ENTERPRISE_INSTALLATION_RUNBOOK.md) for a
fielded delivery. Local credentials, loopback origins, registry settings, acceptance
harness identities, and destructive cleanup commands are not enterprise defaults.
The [enterprise installation readiness record](ENTERPRISE_INSTALLATION_READINESS.md)
lists the decisions and evidence required when translating local results to a customer
environment.

## Source Of Truth

This guide owns the ordered developer workflow. The linked documents remain
authoritative for their individual contracts:

| Concern | Owner |
|---|---|
| Typed local profile contract | [Local deployment profiles](LOCAL_DEPLOYMENT_PROFILES.md) |
| k3d, GPU node, registry, and restart behavior | [Local k3d development](../deploy/local/k3d/README.md) |
| SUMO behavior and acceptance | [SUMO showcase](../showcase/sumo/README.md) |
| Kubernetes chart inputs | [Veoveo Helm installation](../deploy/helm/veoveo/README.md) |
| Production and GitOps boundary | [Enterprise deployment](ENTERPRISE_DEPLOYMENT.md) |

When this guide and an owning contract disagree, stop and resolve the documentation
discrepancy before changing deployment behavior.

## Safety Boundary

The local profile uses loopback HTTP, fixed public development credentials, a local
container registry, and a generated local Keycloak CA. Never reuse these credentials or
trust material in a shared or production cluster.

The lifecycle commands have different data effects:

| Command | Effect on local data |
|---|---|
| `profile-cluster-up` | Creates or reconciles the cluster; preserves existing volumes |
| `profile-up` | Installs or upgrades the selected releases; preserves PVCs |
| `profile-down` | Removes Helm releases; PVC retention depends on the rendered resources |
| `profile-cluster-stop` | Stops the k3d cluster without deleting its volumes |
| `profile-cluster-delete` | Deletes the cluster and can delete every local PVC and recording |

Do not run `profile-cluster-delete` as routine restart recovery.

## Host Prerequisites

The verified host path is Linux with the native Docker Engine socket. Docker must be
able to use the NVIDIA Container Toolkit, and the current user must be able to invoke
Docker without changing daemon context midway through the workflow.

Required host tools are:

- Git and Rust/Cargo;
- Docker Engine with Buildx;
- an accessible NVIDIA GPU, driver, and NVIDIA Container Toolkit;
- `k3d`, `kubectl`, and `helm` at the versions pinned in
  [`deploy/local/k3d/versions.env`](../deploy/local/k3d/versions.env).

Confirm the host before creating anything:

```bash
docker version
docker context show
docker info
docker buildx version
nvidia-smi
```

For a native system daemon, use its canonical socket in the current shell:

```bash
export DOCKER_HOST=unix:///var/run/docker.sock
```

Install the pinned Kubernetes tools using the commands and checksum requirement in
[Local k3d development](../deploy/local/k3d/README.md#tool-versions), then verify that
the shell resolves them:

```bash
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"

command -v cargo
command -v docker
command -v k3d
command -v kubectl
command -v helm

k3d version
kubectl version --client
helm version
```

## Build The GPU Node Image

Build the pinned k3d node image once after installing the prerequisites. Rebuild it when
one of its pinned inputs changes:

```bash
source deploy/local/k3d/versions.env

docker build \
  --build-arg K3S_VERSION="$K3S_VERSION" \
  --build-arg CUDA_VERSION="$CUDA_VERSION" \
  --build-arg NVIDIA_CONTAINER_TOOLKIT_VERSION="$NVIDIA_CONTAINER_TOOLKIT_VERSION" \
  --tag "$VEOVEO_K3D_NODE_IMAGE" \
  deploy/local/k3d/node
```

## Clean Installation

Run the workflow from the repository root. Image publication uses an exact committed
revision. Changes to profile-owned inputs must be committed before publication; do not
work around the revision check by force-adding generated files.

```bash
export PROFILE=showcase/sumo/deploy/deployment.json
export REVISION="$(git rev-parse HEAD)"
export LOCK="/tmp/veoveo-sumo-${REVISION}.lock.json"

git status --short --branch
cargo xtask smoke profile-validate --profile "$PROFILE"
```

Create the cluster and reconcile its local Keycloak instance:

```bash
cargo xtask smoke profile-cluster-up \
  --profile "$PROFILE"
```

Apply the installation-owner development Secrets. This step is mandatory after every
new cluster. Neither `profile-cluster-up` nor `profile-up` creates Secrets:

```bash
kubectl --context k3d-veoveo-sumo apply \
  -f deploy/local/k3d/development-resources.yaml
```

Ensure the managed builder, publish the exact revision, and create a fresh lock:

```bash
cargo xtask image builder ensure

cargo xtask release images \
  --profile "$PROFILE" \
  --profile-revision "$REVISION" \
  --lock-output "$LOCK"

test -s "$LOCK" && echo "Lock created: $LOCK"
```

Install the locked platform and SUMO releases:

```bash
cargo xtask smoke profile-up \
  --profile "$PROFILE" \
  --lock "$LOCK"
```

## Acceptance

First inspect Kubernetes readiness:

```bash
kubectl --context k3d-veoveo-sumo \
  --namespace veoveo get pods

kubectl --context k3d-veoveo-sumo \
  get --raw=/readyz
```

Every long-running pod must reach `Running` and its complete Ready count. Bootstrap and
initialization jobs may remain `Completed`.

Run the composed acceptance rather than treating a reachable web page as sufficient:

```bash
cargo xtask smoke sumo-verify \
  --context k3d-veoveo-sumo
```

Successful acceptance reports both of these outcomes:

```text
Console login, SUMO History archive, and Live RRD stream verified ok
sumo verify ok
```

Open the Console at <http://localhost:8780/console/> and use the disposable local
identity:

```text
username: alice
password: keycloak-local-password
```

The acceptance command, not manual browser reachability alone, proves the local SUMO
workflow.

## Daily Start And Host Restart

After a normal host or Docker restart, do not recreate the cluster. Reconcile the
existing cluster, its Docker network, Kubernetes API, and local Keycloak instance:

```bash
export PROFILE=showcase/sumo/deploy/deployment.json

cargo xtask smoke profile-cluster-up \
  --profile "$PROFILE"

cargo xtask smoke sumo-verify \
  --context k3d-veoveo-sumo
```

If the releases are already installed, `profile-up` is not required merely because the
host restarted.

## Non-Destructive Stop And Start

Stop the cluster while retaining its local volumes:

```bash
cargo xtask smoke profile-cluster-stop \
  --profile showcase/sumo/deploy/deployment.json
```

Start and reconcile it again:

```bash
cargo xtask smoke profile-cluster-up \
  --profile showcase/sumo/deploy/deployment.json
```

## Clean Reset

Use a clean reset only when data loss is acceptable. This deletes the k3d cluster,
Kubernetes resources, generated local Keycloak material, and local persistent data. The
shared registry remains because other profiles may use it.

```bash
export PROFILE=showcase/sumo/deploy/deployment.json

cargo xtask smoke profile-cluster-delete \
  --profile "$PROFILE"
```

Return to [Clean Installation](#clean-installation) afterward. In particular, reapply
`development-resources.yaml`; the deleted Secrets do not survive cluster recreation.

## Failure Diagnosis

Stop at the first failed command. Capture state before deleting resources or changing
code.

### Required tools are not on PATH

Confirm both the files and shell resolution:

```bash
ls -l ~/.local/bin/k3d ~/.local/bin/kubectl ~/.local/bin/helm
printf 'PATH=%s\n' "$PATH"
command -v k3d
command -v kubectl
command -v helm
```

An executable file can exist while the current shell still lacks `~/.local/bin` on
`PATH`.

### The GPU node image cannot be pulled

An error attempting to pull `veoveo/k3s-gpu:...` from Docker Hub means the required
local node image was not built in the Docker daemon currently selected by
`DOCKER_HOST`. Return to [Build The GPU Node Image](#build-the-gpu-node-image).

### Secret-reference closure reports missing Secrets

For a newly created loopback cluster, apply the checked-in fixture and retry
`profile-up`:

```bash
kubectl --context k3d-veoveo-sumo apply \
  -f deploy/local/k3d/development-resources.yaml
```

Do not copy this remedy to a shared or production cluster.

### Kubernetes returns EOF or ServiceUnavailable after restart

Record the state first:

```bash
k3d cluster list
docker inspect k3d-veoveo-sumo-server-0
docker network inspect k3d-veoveo-sumo
docker logs k3d-veoveo-sumo-server-0
```

Then attempt an ordered, non-destructive restart:

```bash
cargo xtask smoke profile-cluster-stop --profile "$PROFILE"
cargo xtask smoke profile-cluster-up --profile "$PROFILE"
```

Only use `profile-cluster-delete` after the command reports that bounded recovery failed
and after accepting the loss of local PVC data. The detailed network failure signature
is documented in [Local k3d development](../deploy/local/k3d/README.md#troubleshooting-after-a-host-or-docker-restart).

### A workload misses its readiness deadline

Inspect the failing workload rather than immediately reinstalling:

```bash
kubectl --context k3d-veoveo-sumo --namespace veoveo get pods
kubectl --context k3d-veoveo-sumo --namespace veoveo describe pod <pod-name>
kubectl --context k3d-veoveo-sumo --namespace veoveo logs <pod-name> -c <container>
kubectl --context k3d-veoveo-sumo --namespace veoveo logs <pod-name> -c <container> --previous
```

The current and previous container logs distinguish image, configuration, dependency,
authentication, and crash-loop failures.

### The node reports DiskPressure or Pods remain Pending

Kubernetes API readiness does not imply that workloads can schedule. Inspect the node,
taints, events, filesystems, Docker inventory, and the k3d node runtime before deleting
anything:

```bash
kubectl --context k3d-veoveo-sumo get nodes -o wide
kubectl --context k3d-veoveo-sumo describe node
kubectl --context k3d-veoveo-sumo get events --all-namespaces \
  --sort-by=.lastTimestamp
df -h
df -i
docker system df
docker exec k3d-veoveo-sumo-server-0 df -h
docker exec k3d-veoveo-sumo-server-0 df -i
docker exec k3d-veoveo-sumo-server-0 crictl images --digests
```

Distinguish node filesystem capacity, image garbage collection, Docker image storage,
BuildKit cache, containerd content, logs, and persistent volumes. A cache reported as
reclaimable may belong to a different BuildKit daemon than the selected builder.

Do not prune images, caches, containers, volumes, or Kubernetes resources until the
owning store and protected data have been identified. PVCs, k3d volumes, databases,
active images, and generated credential roots are not cleanup candidates merely because
the node has disk pressure.

### A Pod sandbox or external image cannot be pulled

Record the exact image reference and first causal error. Confirm DNS and registry access
from the k3d node, then verify the image in the container runtime actually used by
kubelet. An image present in the Docker host store is not proof that containerd can
resolve it under the required reference.

Use `crictl inspecti <exact-image-reference>` inside the k3d server container to prove
that kubelet's runtime can resolve the image. Preserve the registry prefix, repository,
tag or digest, platform, and containerd namespace shown by the failing event.

Do not turn a one-time image import into the permanent remedy for broken node DNS. The
durable correction belongs to the local cluster networking or registry-mirror owner.

## Agent Workflow

Agents working on local deployment must:

1. Read this guide, [Local deployment profiles](LOCAL_DEPLOYMENT_PROFILES.md), and the
   profile-specific showcase README before acting.
2. Inspect the current branch, worktree status, Docker context, cluster inventory, and
   Kubernetes readiness before proposing a repair.
3. Treat local Keycloak, loopback HTTP, and `development-resources.yaml` as local-only
   fixtures. They never become production requirements by inference.
4. Diagnose an observed failure before editing code. A failed local composition does
   not authorize a new repository-wide production rule.
5. Stop when an installation command fails unless the user explicitly authorizes the
   diagnosed repair.
6. Report exactly which acceptance was executed. Focused unit tests, a reachable
   Console, and the complete `sumo-verify` scenario are different evidence.
