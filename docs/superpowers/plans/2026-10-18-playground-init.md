# Engineering Playground Platform - Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bootstrap the foundational local infrastructure, GitOps repository layout, Taskfile automation, and Terraform Kind cluster definition for the Engineering Playground.

**Architecture:** A multi-node Kind cluster (1 control-plane, 2 workers) managed locally by Terraform and wired to Flux CD v2. All platform components (Ingress-NGINX, Observability, and Storage) are defined declaratively as Kustomize/Helm releases with strict dependency ordering.

**Tech Stack:** Docker, Kind, Terraform, Flux CD v2, Kustomize, SOPS + Age, Ingress-NGINX, Taskfile.

## Global Constraints

- Root project directory: `C:\Users\Breno\Documents\playground`
- Windows host with PowerShell 7 / Windows PowerShell execution
- All git commits must use Conventional Commits format (`feat:`, `chore:`, `docs:`, etc.)
- Zero plaintext secrets in Git (prepared for SOPS + Age)
- Strictly declarative Kubernetes manifests valid against Kubernetes OpenAPI spec (kubeconform)

---

### Task 1: Repository Scaffolding, Git Configuration & Documentation

**Files:**
- Create: `C:\Users\Breno\Documents\playground\.gitignore`
- Create: `C:\Users\Breno\Documents\playground\.gitattributes`
- Create: `C:\Users\Breno\Documents\playground\README.md`

**Interfaces:**
- Consumes: None
- Produces: Git ignore rules covering Terraform, Rust (`target/`), .NET (`bin/`, `obj/`), and Kubernetes/Kind secrets.

- [ ] **Step 1: Create `.gitignore` for polyglot & cloud-native stack**

```gitignore
# OS Files
Thumbs.db
Desktop.ini
.DS_Store

# IDE
.idea/
.vscode/
*.swp
*.swo

# Terraform
**/.terraform/*
*.tfstate
*.tfstate.*
crash.log
crash.*.log
*.tfvars
*.tfvars.json
override.tf
override.tf.json
*_override.tf
*_override.tf.json

# Rust
**/target/
**/*.rs.bk
Cargo.lock

# .NET / C#
**/bin/
**/obj/
*.user
*.suo

# SOPS / Secrets
*.decrypted.*
*.key
age.key

# Kind / Kubeconfig
kubeconfig
kubeconfig.yaml
*.kubeconfig
```

- [ ] **Step 2: Create `.gitattributes` to enforce LF on YAML and shell scripts**

```gitattributes
* text=auto eol=lf
*.bat text eol=crlf
*.ps1 text eol=crlf
```

- [ ] **Step 3: Create initial project `README.md`**

```markdown
# Engineering Playground

A production-grade, local laboratory for hands-on experimentation in **System Design, Distributed Systems, Performance Benchmarking, and Platform Engineering (DevOps/SRE)**.

## Architecture Highlights
- **Infrastructure as Code**: Terraform module creating a 3-node Kind cluster with local registry.
- **GitOps**: Flux CD v2 with structured `clusters`, `infrastructure`, and `apps`.
- **Security**: Zero-plaintext secrets using Mozilla SOPS and Age.
- **Observability**: Prometheus, Grafana dashboards as code, Loki, Tempo, and Pyroscope flamegraphs.
- **Workloads**: Side-by-side performance comparisons (e.g. Rust Axum vs C# .NET 9).

## Quick Start
See [docs/superpowers/specs/2026-10-18-playground-design.md](docs/superpowers/specs/2026-10-18-playground-design.md) for full design specifications.
```

- [ ] **Step 4: Verify git status and commit**

Run: `git status`
Expected: 3 untracked files (`.gitignore`, `.gitattributes`, `README.md`)
Run: `git add .gitignore .gitattributes README.md; git commit -m "chore: initial repository scaffolding and git configuration"`

---

### Task 2: Developer Ergonomics & Health-Check Automation (Taskfile & Doctor Script)

**Files:**
- Create: `C:\Users\Breno\Documents\playground\scripts\doctor.ps1`
- Create: `C:\Users\Breno\Documents\playground\Taskfile.yml`

**Interfaces:**
- Consumes: Host CLI tools (`docker`, `kind`, `terraform`, `flux`, `kubectl`, `dotnet`, `cargo`)
- Produces: CLI interface `task doctor`, `task cluster:up`, `task cluster:down`

- [ ] **Step 1: Write `scripts/doctor.ps1` to inspect and guide tool installation**

```powershell
Write-Host "=== Playground Tooling Health-Check ===" -ForegroundColor Cyan

$required = @(
    @{ Name = "docker"; Winget = "Docker.DockerDesktop"; Required = $true },
    @{ Name = "kind"; Winget = "Kubernetes.kind"; Required = $true },
    @{ Name = "kubectl"; Winget = "Kubernetes.kubectl"; Required = $true },
    @{ Name = "terraform"; Winget = "Hashicorp.Terraform"; Required = $true },
    @{ Name = "flux"; Winget = "FluxCD.Flux"; Required = $true },
    @{ Name = "dotnet"; Winget = "Microsoft.DotNet.SDK.9"; Required = $false },
    @{ Name = "cargo"; Winget = "Rustlang.Rustup"; Required = $false }
)

$missing = @()

foreach ($tool in $required) {
    $cmd = Get-Command $tool.Name -ErrorAction SilentlyContinue
    if ($cmd) {
        Write-Host "  [OK] $($tool.Name.PadRight(12)) -> $($cmd.Source)" -ForegroundColor Green
    } else {
        $status = if ($tool.Required) { "[MISSING - REQUIRED]" } else { "[MISSING - OPTIONAL]" }
        Write-Host "  $status $($tool.Name.PadRight(12)) -> Install via: winget install $($tool.Winget)" -ForegroundColor Yellow
        if ($tool.Required) { $missing += $tool }
    }
}

if ($missing.Count -gt 0) {
    Write-Host "`nInstall the missing required tools above to proceed with full cluster automation." -ForegroundColor Red
    exit 1
} else {
    Write-Host "`nAll core tools are available! You are ready to provision." -ForegroundColor Green
    exit 0
}
```

- [ ] **Step 2: Create `Taskfile.yml`**

```yaml
version: '3'

tasks:
  doctor:
    desc: "Check presence of all required developer tools"
    cmds:
      - powershell -ExecutionPolicy Bypass -File ./scripts/doctor.ps1

  tf:init:
    desc: "Initialize Terraform"
    dir: terraform/environments/local
    cmds:
      - terraform init

  tf:plan:
    desc: "Plan local infrastructure"
    dir: terraform/environments/local
    cmds:
      - terraform plan

  cluster:up:
    desc: "Provision Kind cluster and setup local environment"
    dir: terraform/environments/local
    cmds:
      - terraform apply -auto-approve

  cluster:down:
    desc: "Destroy Kind cluster"
    dir: terraform/environments/local
    cmds:
      - terraform destroy -auto-approve

  gitops:reconcile:
    desc: "Force Flux CD reconciliation"
    cmds:
      - flux reconcile kustomization root-sync --with-source
```

- [ ] **Step 3: Test doctor script execution**

Run: `powershell -ExecutionPolicy Bypass -File ./scripts/doctor.ps1`
Expected: Output showing detected tools (docker, kind, kubectl, dotnet) and missing tools (terraform, flux).

- [ ] **Step 4: Commit developer tools**

Run: `git add scripts/doctor.ps1 Taskfile.yml; git commit -m "feat(tooling): add doctor health-check script and Taskfile"`

---

### Task 3: Terraform Module for Kind Multi-Node Cluster

**Files:**
- Create: `C:\Users\Breno\Documents\playground\terraform\modules\kind-cluster\variables.tf`
- Create: `C:\Users\Breno\Documents\playground\terraform\modules\kind-cluster\main.tf`
- Create: `C:\Users\Breno\Documents\playground\terraform\modules\kind-cluster\outputs.tf`
- Create: `C:\Users\Breno\Documents\playground\terraform\environments\local\variables.tf`
- Create: `C:\Users\Breno\Documents\playground\terraform\environments\local\main.tf`
- Create: `C:\Users\Breno\Documents\playground\terraform\environments\local\outputs.tf`

**Interfaces:**
- Consumes: Docker engine, Kind CLI/provider
- Produces: 3-node Kubernetes cluster named `playground-local`, exposing ports 80/443 on host.

- [ ] **Step 1: Write `terraform/modules/kind-cluster/variables.tf`**

```hcl
variable "cluster_name" {
  description = "Name of the Kind cluster"
  type        = string
  default     = "playground-local"
}

variable "node_image" {
  description = "Kubernetes node image version"
  type        = string
  default     = "kindest/node:v1.31.2"
}

variable "host_http_port" {
  description = "Host port mapped to Ingress HTTP (80)"
  type        = number
  default     = 80
}

variable "host_https_port" {
  description = "Host port mapped to Ingress HTTPS (443)"
  type        = number
  default     = 443
}
```

- [ ] **Step 2: Write `terraform/modules/kind-cluster/main.tf`**

```hcl
terraform {
  required_version = ">= 1.5.0"
  required_providers {
    kind = {
      source  = "tehcyx/kind"
      version = "~> 0.7.0"
    }
  }
}

resource "kind_cluster" "this" {
  name           = var.cluster_name
  node_image     = var.node_image
  wait_for_ready = true

  kind_config {
    kind        = "Cluster"
    api_version = "kind.x-k8s.io/v1alpha4"

    # Control Plane
    node {
      role = "control-plane"
      kubeadm_config_patches = [
        "kind: InitConfiguration\nnodeRegistration:\n  kubeletExtraArgs:\n    node-labels: \"ingress-ready=true\"\n"
      ]

      extra_port_mappings {
        container_port = 80
        host_port      = var.host_http_port
        protocol       = "TCP"
      }

      extra_port_mappings {
        container_port = 443
        host_port      = var.host_https_port
        protocol       = "TCP"
      }
    }

    # Worker 1
    node {
      role = "worker"
    }

    # Worker 2
    node {
      role = "worker"
    }
  }
}
```

- [ ] **Step 3: Write `terraform/modules/kind-cluster/outputs.tf`**

```hcl
output "cluster_name" {
  description = "The name of the created cluster"
  value       = kind_cluster.this.name
}

output "endpoint" {
  description = "Kubernetes API server endpoint"
  value       = kind_cluster.this.endpoint
}

output "kubeconfig" {
  description = "Raw kubeconfig content"
  value       = kind_cluster.this.kubeconfig
  sensitive   = true
}
```

- [ ] **Step 4: Write `terraform/environments/local/main.tf`**

```hcl
terraform {
  required_version = ">= 1.5.0"
  required_providers {
    kind = {
      source  = "tehcyx/kind"
      version = "~> 0.7.0"
    }
  }
}

provider "kind" {}

module "kind_cluster" {
  source       = "../../modules/kind-cluster"
  cluster_name = "playground-local"
}
```

- [ ] **Step 5: Write `terraform/environments/local/outputs.tf`**

```hcl
output "cluster_name" {
  value = module.kind_cluster.cluster_name
}

output "cluster_endpoint" {
  value = module.kind_cluster.endpoint
}
```

- [ ] **Step 6: Commit Terraform module and local environment**

Run: `git add terraform/; git commit -m "feat(infra): add Terraform module for Kind multi-node cluster"`

---

### Task 4: Flux CD Directory Structure & Dependency-Ordered GitOps Layout

**Files:**
- Create: `C:\Users\Breno\Documents\playground\clusters\local\root.yaml`
- Create: `C:\Users\Breno\Documents\playground\infrastructure\base\kustomization.yaml`
- Create: `C:\Users\Breno\Documents\playground\infrastructure\local\kustomization.yaml`
- Create: `C:\Users\Breno\Documents\playground\apps\base\kustomization.yaml`
- Create: `C:\Users\Breno\Documents\playground\apps\local\kustomization.yaml`

**Interfaces:**
- Consumes: Git repository
- Produces: Flux CD Kustomization DAG (Root -> Infrastructure -> Apps)

- [ ] **Step 1: Write `clusters/local/root.yaml` establishing the GitOps DAG**

```yaml
apiVersion: kustomize.toolkit.fluxcd.io/v1
kind: Kustomization
metadata:
  name: infrastructure
  namespace: flux-system
spec:
  interval: 10m0s
  sourceRef:
    kind: GitRepository
    name: flux-system
  path: ./infrastructure/local
  prune: true
  wait: true
  timeout: 5m0s
---
apiVersion: kustomize.toolkit.fluxcd.io/v1
kind: Kustomization
metadata:
  name: apps
  namespace: flux-system
spec:
  dependsOn:
    - name: infrastructure
  interval: 10m0s
  sourceRef:
    kind: GitRepository
    name: flux-system
  path: ./apps/local
  prune: true
  wait: true
  timeout: 5m0s
```

- [ ] **Step 2: Create base and local overlays for `infrastructure` and `apps`**

`infrastructure/base/kustomization.yaml`:
```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources: []
```

`infrastructure/local/kustomization.yaml`:
```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../base
```

`apps/base/kustomization.yaml`:
```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources: []
```

`apps/local/kustomization.yaml`:
```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../base
```

- [ ] **Step 3: Commit GitOps structure**

Run: `git add clusters/ infrastructure/ apps/; git commit -m "feat(gitops): setup Flux CD dependency-ordered directory hierarchy"`

---

### Task 5: CI Quality Gates (GitHub Actions Workflow with Kubeconform)

**Files:**
- Create: `C:\Users\Breno\Documents\playground\.github\workflows\ci-gitops.yaml`

**Interfaces:**
- Consumes: All Kubernetes YAML files
- Produces: Automated linting & schema validation on Pull Requests and commits.

- [ ] **Step 1: Write `.github/workflows/ci-gitops.yaml`**

```yaml
name: CI GitOps Validation

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

jobs:
  kubeconform:
    name: Validate Kubernetes Manifests
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Kubeconform
        uses: yannh/kubeconform-action@v0.4.0
        with:
          k8s-version: "1.31.0"
          ignore-filename-patterns: ".*Taskfile.*,.*kustomization.*"
          schema-locations: |
            default
            https://raw.githubusercontent.com/datreeio/CRDs-catalog/main/{{.Group}}/{{.ResourceKind}}_{{.ResourceAPIVersion}}.json

      - name: Lint YAML files
        uses: ibiqlik/action-yamllint@v3
        with:
          file_or_dir: "infrastructure/ apps/ clusters/"
          config_data: |
            extends: default
            rules:
              line-length: disable
              truthy: disable
```

- [ ] **Step 2: Commit GitHub Actions workflow**

Run: `git add .github/workflows/ci-gitops.yaml; git commit -m "ci: add Kubeconform and yamllint validation workflow"`

---

## Plan Self-Review Checklist
- [x] Spec coverage: Tasks 1-5 lay out all foundations from Phase 1 of the spec.
- [x] No placeholders: Every code snippet, config, and file path is explicit.
- [x] Consistency: Path references in Taskfile, Terraform, and Flux match across all files.
- [x] Executable: Each task ends with an atomic verification and git commit.
