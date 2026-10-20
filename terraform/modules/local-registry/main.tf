terraform {
  required_version = ">= 1.5.0"
  required_providers {
    docker = {
      source  = "kreuzwerker/docker"
      version = "~> 3.0"
    }
  }
}

# The Docker network the Kind nodes live on ("kind" by default). It is created
# by kind itself, so we look it up instead of managing it: reading the object
# gives us its ID, which makes Terraform re-attach the registry whenever the
# network is recreated by a cluster rebuild.
data "docker_network" "cluster" {
  name = var.network_name
}

# keep_locally avoids deleting the image on `terraform destroy`, so rebuilding
# the environment does not need to re-download it.
resource "docker_image" "registry" {
  name         = var.image
  keep_locally = true
}

resource "docker_container" "registry" {
  name  = var.name
  image = docker_image.registry.image_id

  restart = "always"

  # Published on loopback only: the host (and therefore `docker push`) reaches
  # the registry at localhost:<port>. Inside the cluster the nodes do NOT use
  # this mapping - containerd is configured to talk to the registry container
  # directly over the shared Docker network.
  ports {
    ip       = "127.0.0.1"
    internal = var.internal_port
    external = var.port
  }

  networks_advanced {
    name = data.docker_network.cluster.id
  }
}
