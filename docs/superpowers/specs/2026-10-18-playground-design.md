# Engineering Playground - Architecture & Design Spec

- **Author**: Breno & Antigravity
- **Date**: 2026-10-18
- **Status**: Approved
- **Target Audience**: Senior DevOps / Platform / Distributed Systems Engineers
- **Location**: `C:\Users\Breno\Documents\playground`

---

## 1. Executive Summary & Vision

This repository is designed as a production-grade, reproducible **Engineering Playground** for hands-on experimentation in **System Design, Distributed Systems, Performance Benchmarking, and Platform Engineering (DevOps/SRE)**.

Rather than relying on toy setups or ad-hoc scripts, the platform adopts real-world enterprise standards:
- **Infrastructure as Code (IaC)** with Terraform to bootstrap the local environment.
- **Declarative GitOps** with Flux CD v2 following CNCF best practices (`clusters`, `infrastructure`, `apps` with strict `dependsOn` directed acyclic graphs).
- **Zero-Trust GitOps Secrets** using Mozilla SOPS and Age encryption.
- **Production Observability & Continuous Profiling** (Prometheus, Grafana, Loki, Tempo, and Pyroscope flamegraphs).
- **Polyglot Workload Benchmarks** comparing Rust (Axum/Tokio) and C# (ASP.NET Core 9) under identical resource limits.
- **Database & Storage Labs** (PostgreSQL B-Tree vs BRIN vs Hash indexes with millions of rows).
- **Scientific Experimentation** backed by Architecture Decision Records (ADRs) and k6 automated load suites.

All components run locally on an x86_64 host (targeting 16 CPU cores and 32GB DDR5 RAM).

---

## 2. Architectural Blueprint

```
+-----------------------------------------------------------------------------------------+
| HOST (Windows / WSL2 - 32GB DDR5, 16 Cores)                                             |
|                                                                                         |
|  [Terraform (tehcyx/kind + fluxcd/flux)]                                                |
|       |                                                                                 |
|       +---> Provisions Kind Cluster + Bootstraps Flux CD GitOps                         |
|                                                                                         |
|  [Kind Multi-Node Cluster]                                                              |
|  +--------------------------+  +-----------------------------------------------------+  |
|  | Node: Control-Plane      |  | Nodes: Worker 1 & Worker 2                          |  |
|  | - kube-apiserver         |  | - Ingress-NGINX (Port 80/443 mapped to Host)        |  |
|  | - etcd                   |  | - Observability: Prometheus, Grafana, Tempo,        |  |
|  | - Flux CD Controllers   |  |                  Loki, Pyroscope                    |  |
|  |   (Source, Kustomize,    |  | - Data Layer: PostgreSQL, Redis                     |  |
|  |    Helm, Notification)   |  | - Apps: `api-rust` (Axum) & `api-csharp` (ASP.NET 9)|  |
|  +--------------------------+  +-----------------------------------------------------+  |
|                                                                                         |
|  [Local Container Registry] -> localhost:5001                                           |
|  [Load Testing] -----------> k6 / oha targeting Ingress on localhost                    |
+-----------------------------------------------------------------------------------------+
```

---

## 3. Technology Stack & Decision Matrix

| Domain | Technology | Justification & Senior Trade-off |
|---|---|---|
| **Cluster Engine** | **Kind (Kubernetes in Docker)** | Upstream Kubernetes standard. Perfectly simulates multi-node scheduling, node drains, and anti-affinity without cloud costs. |
| **IaC** | **Terraform** | Enforces the clear separation between "Infrastructure Provisioning" (the cluster) and "Workload GitOps" (inside the cluster). |
| **GitOps** | **Flux CD v2** | CNCF graduated, enterprise-grade, native CRD-based, headless, zero UI bloat. |
| **Secret Management** | **Mozilla SOPS + Age** | Eliminates plaintext secrets in Git. Decentralized decryption inside the cluster without requiring a heavy HashiCorp Vault server. |
| **Ingress / L7** | **Ingress-NGINX** | Lightweight, rock solid, standard ingress metrics. Avoids early Istio sidecar memory/CPU penalties. |
| **Observability** | **kube-prometheus-stack** | Prometheus Operator, Node Exporter, kube-state-metrics, Grafana dashboards as code. |
| **Tracing & Logs** | **Tempo + Loki** | Grafana LGTM stack; lightweight, native Grafana integration, correlates logs and traces via `trace_id`. |
| **Profiling** | **Grafana Pyroscope** | Continuous CPU and memory profiling. Enables flamegraphs to diagnose GC pauses (.NET) vs allocations (Rust). |
| **Workloads** | **Rust (Axum) vs C# (.NET 9)** | Direct comparison of GC-managed runtime vs compile-time memory model under heavy concurrency. |
| **Benchmarking** | **k6 & oha** | Scriptable load testing with custom metrics (p95, p99, error rate, throughput saturation). |
| **Task Runner** | **Taskfile (Go-task)** | Modern, cross-platform alternative to Makefile. Clear, reproducible developer commands. |

---

## 4. Repository Layout & Directory Taxonomy

```text
playground/
├── .github/
│   └── workflows/
│       ├── ci-gitops.yaml       # Kubeconform schema validation & yamllint
│       ├── ci-apps.yaml         # cargo test, dotnet test, docker build & trivy scan
│       └── adr-lint.yaml        # Ensures experiments follow standard template
│
├── Taskfile.yml                 # Unified developer CLI (task up, task bench, etc.)
│
├── terraform/                   # Infrastructure as Code
│   ├── modules/
│   │   └── kind-cluster/        # Kind cluster configuration & local registry hook
│   └── environments/
│       └── local/               # Terraform root module (Kind + Flux bootstrap)
│           ├── main.tf
│           ├── variables.tf
│           └── outputs.tf
│
├── clusters/
│   └── local/                   # Flux GitOps entrypoint for local cluster
│       ├── flux-system/         # Autogenerated Flux manifests
│       └── root.yaml            # Master Kustomization pointing to infrastructure & apps
│
├── infrastructure/              # Platform infrastructure managed by Flux
│   ├── base/
│   │   ├── ingress-nginx/       # HelmRelease & values
│   │   ├── observability/       # Prometheus, Grafana, Tempo, Loki, Pyroscope
│   │   └── storage/             # PostgreSQL StatefulSet & Redis
│   └── local/                   # Kustomize overlays for local resource allocation
│
├── apps/                        # Application workloads managed by Flux
│   ├── api-rust/
│   │   ├── base/                # Deployment, Service, HPA, PDB, ServiceMonitor
│   │   └── overlays/local/
│   └── api-csharp/
│       ├── base/                # Deployment, Service, HPA, PDB, ServiceMonitor
│       └── overlays/local/
│
├── services/                    # Application source code
│   ├── api-rust/                # Axum + Tokio + SQLx + OpenTelemetry
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   └── Dockerfile
│   └── api-csharp/              # ASP.NET Core 9 + Dapper + OpenTelemetry
│       ├── ApiCSharp.csproj
│       ├── Program.cs
│       └── Dockerfile
│
├── experiments/                 # Scientific system design labs
│   ├── 01-rust-vs-csharp-throughput/
│   │   ├── ADR.md               # Hypothesis, methodology, conclusions
│   │   ├── k6-script.js         # Reproducible load test
│   │   └── results/             # Flamegraphs, Prometheus metrics snapshots
│   ├── 02-postgres-indexing-benchmark/
│   │   ├── ADR.md               # B-Tree vs BRIN vs Hash under 10M rows
│   │   ├── seed.sql
│   │   └── queries.sql
│   └── 03-resilience-graceful-shutdown/
│       ├── ADR.md               # Zero-downtime deployment verification under traffic
│       └── chaos-test.sh
│
└── docs/                        # Project documentation, guides and architecture records
    └── superpowers/specs/       # Validated design specifications
```

---

## 5. Senior Reliability & Production Readiness Standards

All workload deployments (`apps/*`) must adhere to:

1. **Pod Disruption Budgets (PDB)**: `minAvailable: 1` or `maxUnavailable: 1` to prevent complete service downtime during node rolling updates.
2. **Deterministic Lifecycle Hooks**:
   - `preStop`: `sleep 5` hook allowing Ingress endpoints to deregister the pod before connection termination.
   - `terminationGracePeriodSeconds`: Set to 30-45s for draining in-flight requests.
3. **Tri-Level Health Probes**:
   - `startupProbe`: Protects slow starting instances without artificially increasing failure thresholds.
   - `livenessProbe`: Recovers hung pods.
   - `readinessProbe`: Removes unready pods from the Ingress endpoints.
4. **Hard Security Contexts**:
   - `runAsNonRoot: true`
   - `allowPrivilegeEscalation: false`
   - `readOnlyRootFilesystem: true`
   - `drop: ["ALL"]` capabilities
5. **Quality of Service (QoS)**: Explicit `requests` and `limits` matching for latency-critical pods (Guaranteed QoS) to prevent CPU throttling and OOM kills.
