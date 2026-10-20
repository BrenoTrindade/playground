output "name" {
  description = "Container name of the registry"
  value       = docker_container.registry.name
}

output "host" {
  description = "Host:port to use as the image prefix when tagging and pushing"
  value       = "localhost:${var.port}"
}
