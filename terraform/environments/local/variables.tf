variable "cluster_name" {
  description = "Name of the local playground cluster"
  type        = string
  default     = "playground-local"
}

variable "registry_container_name" {
  description = "Name of the Docker container running the local image registry"
  type        = string
  default     = "kind-registry"
}

variable "registry_image" {
  description = "Pinned registry image"
  type        = string
  default     = "registry:2.8.3"
}

variable "registry_network_name" {
  description = "Docker network shared between the Kind nodes and the registry"
  type        = string
  default     = "kind"
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
