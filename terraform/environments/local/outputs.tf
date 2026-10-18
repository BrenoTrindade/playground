output "cluster_name" {
  description = "The name of the provisioned Kind cluster"
  value       = module.kind_cluster.cluster_name
}

output "cluster_endpoint" {
  description = "The Kubernetes API server endpoint"
  value       = module.kind_cluster.endpoint
}
