terraform {
  required_version = ">= 1.5.0"
  required_providers {
    kind = {
      source  = "tehcyx/kind"
      version = "~> 0.7.0"
    }
  }
}

provider "kind" {}

module "kind_cluster" {
  source       = "../../modules/kind-cluster"
  cluster_name = var.cluster_name
}
