variable "name" {
  description = "Name of the Docker container running the registry"
  type        = string
  default     = "kind-registry"
}

variable "image" {
  description = "Pinned registry image"
  type        = string
  default     = "registry:2.8.3"
}

variable "network_name" {
  description = "Docker network shared with the Kind nodes"
  type        = string
  default     = "kind"
}

variable "port" {
  description = "Host port the registry is published on (image prefix: localhost:<port>)"
  type        = number
  default     = 5001
}

variable "internal_port" {
  description = "Port the registry listens on inside the container"
  type        = number
  default     = 5000
}
