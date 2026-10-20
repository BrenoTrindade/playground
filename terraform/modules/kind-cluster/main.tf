terraform {
  required_version = ">= 1.5.0"
  required_providers {
    kind = {
      source  = "tehcyx/kind"
      version = "~> 0.7.0"
    }
  }
}

resource "kind_cluster" "this" {
  name           = var.cluster_name
  node_image     = var.node_image
  wait_for_ready = true

  kind_config {
    kind        = "Cluster"
    api_version = "kind.x-k8s.io/v1alpha4"

    # Local container registry support (official Kind pattern).
    #
    # Images are referenced as `localhost:<port>/<image>:<tag>`, both from the
    # host and from inside the cluster. Each node's containerd is told to
    # rewrite that reference to the registry container, which lives on the
    # same Docker network as the nodes. Plain HTTP is fine here: the registry
    # is only reachable over loopback / the Docker network.
    containerd_config_patches = [
      <<-TOML
      [plugins."io.containerd.grpc.v1.cri".registry.mirrors."localhost:${var.registry_port}"]
        endpoint = ["http://${var.registry_container_name}:${var.registry_internal_port}"]
      TOML
    ]

    # Control Plane
    node {
      role = "control-plane"
      kubeadm_config_patches = [
        "kind: InitConfiguration\nnodeRegistration:\n  kubeletExtraArgs:\n    node-labels: \"ingress-ready=true\"\n"
      ]

      extra_port_mappings {
        container_port = 80
        host_port      = var.host_http_port
        protocol       = "TCP"
      }

      extra_port_mappings {
        container_port = 443
        host_port      = var.host_https_port
        protocol       = "TCP"
      }
    }

    # Worker 1
    node {
      role = "worker"
    }

    # Worker 2
    node {
      role = "worker"
    }
  }
}
