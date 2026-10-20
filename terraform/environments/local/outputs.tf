output "cluster_name" {
  description = "The name of the provisioned Kind cluster"
  value       = module.kind_cluster.cluster_name
}

output "cluster_endpoint" {
  description = "The Kubernetes API server endpoint"
  value       = module.kind_cluster.endpoint
}

output "registry_endpoint" {
  description = "Image prefix for the local registry (use it to tag images before pushing)"
  value       = "localhost:${var.registry_port}"
}
