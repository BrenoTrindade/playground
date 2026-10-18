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
