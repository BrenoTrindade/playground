# Engineering Playground

A production-grade, local laboratory for hands-on experimentation in **System Design, Distributed Systems, Performance Benchmarking, and Platform Engineering (DevOps/SRE)**.

## Architecture Highlights
- **Infrastructure as Code**: Terraform module creating a 3-node Kind cluster with local registry support.
- **GitOps**: Flux CD v2 with structured `clusters`, `infrastructure`, and `apps`.
- **Security**: Zero-plaintext secrets using Mozilla SOPS and Age.
- **Observability**: Prometheus, Grafana dashboards as code, Loki, Tempo, and Pyroscope flamegraphs.
- **Workloads**: Side-by-side performance comparisons (e.g. Rust Axum vs C# .NET 9).

## Quick Start

### 1. Check Prerequisites
```bash
task doctor
```

### 2. Provision Cluster
```bash
task cluster:up
```

### 3. Reconcile GitOps
```bash
task gitops:reconcile
```

See [docs/superpowers/specs/2026-10-18-playground-design.md](docs/superpowers/specs/2026-10-18-playground-design.md) for full design specifications.
