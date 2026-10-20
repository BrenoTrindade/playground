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

variable "registry_container_name" {
  description = "Name of the Docker container running the local image registry"
  type        = string
  default     = "kind-registry"
}

variable "registry_internal_port" {
  description = "Port the registry listens on inside its container"
  type        = number
  default     = 5000
}

variable "registry_port" {
  description = "Host port exposing the registry, used as the image prefix (localhost:<port>/<image>)"
  type        = number
  default     = 5001
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
