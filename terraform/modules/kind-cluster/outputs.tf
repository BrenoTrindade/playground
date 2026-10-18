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
