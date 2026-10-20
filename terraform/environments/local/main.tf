terraform {
  required_version = ">= 1.5.0"
  required_providers {
    kind = {
      source  = "tehcyx/kind"
      version = "~> 0.7.0"
    }
    docker = {
      source  = "kreuzwerker/docker"
      version = "~> 3.0"
    }
  }
}

provider "kind" {}

provider "docker" {}

module "kind_cluster" {
  source                  = "../../modules/kind-cluster"
  cluster_name            = var.cluster_name
  registry_container_name = var.registry_container_name
  registry_internal_port  = var.registry_internal_port
  registry_port           = var.registry_port
}

# Local image registry. The container runs on the same Docker network as the
# Kind nodes, so a plain HTTP pull from containerd is enough. depends_on keeps
# the network lookup below from running before kind has created it.
module "local_registry" {
  source        = "../../modules/local-registry"
  name          = var.registry_container_name
  image         = var.registry_image
  network_name  = var.registry_network_name
  port          = var.registry_port
  internal_port = var.registry_internal_port

  depends_on = [module.kind_cluster]
}
