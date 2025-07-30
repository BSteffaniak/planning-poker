# DigitalOcean Kubernetes cluster
data "digitalocean_kubernetes_versions" "latest" {
  version_prefix = "1.31."
}

resource "digitalocean_kubernetes_cluster" "planning_poker" {
  name    = "planning-poker-${terraform.workspace}"
  region  = var.digitalocean_region
  version = data.digitalocean_kubernetes_versions.latest.latest_version

  node_pool {
    name       = "planning-poker-pool"
    size       = var.digitalocean_node_size
    node_count = var.digitalocean_node_count
    auto_scale = var.digitalocean_auto_scale
    min_nodes  = var.digitalocean_min_nodes
    max_nodes  = var.digitalocean_max_nodes
  }

  tags = [
    "environment:${terraform.workspace}",
    "project:planning-poker",
    "managed-by:terraform"
  ]

  lifecycle {
    prevent_destroy = true
  }
}

# DigitalOcean Container Registry
resource "digitalocean_container_registry" "planning_poker" {
  name                   = "planning-poker"
  subscription_tier_slug = var.container_registry_tier
  region                 = "nyc3"
}

# Container registry credentials for Kubernetes
resource "digitalocean_container_registry_docker_credentials" "planning_poker" {
  registry_name = digitalocean_container_registry.planning_poker.name
  write         = true
}

# Kubernetes provider using the cluster
provider "kubernetes" {
  host  = digitalocean_kubernetes_cluster.planning_poker.endpoint
  token = digitalocean_kubernetes_cluster.planning_poker.kube_config[0].token
  cluster_ca_certificate = base64decode(
    digitalocean_kubernetes_cluster.planning_poker.kube_config[0].cluster_ca_certificate
  )
}

# Helm provider for cert-manager and other charts
provider "helm" {
  kubernetes {
    host  = digitalocean_kubernetes_cluster.planning_poker.endpoint
    token = digitalocean_kubernetes_cluster.planning_poker.kube_config[0].token
    cluster_ca_certificate = base64decode(
      digitalocean_kubernetes_cluster.planning_poker.kube_config[0].cluster_ca_certificate
    )
  }
}

# Local values for Kubernetes
locals {
  k8s_namespace = "planning-poker-${terraform.workspace}"
  app_name      = "planning-poker"
  app_port      = 8080

  # Container image
  container_image = "${digitalocean_container_registry.planning_poker.endpoint}/${local.app_name}:${var.image_tag}"

  # Common labels
  k8s_labels = {
    app         = local.app_name
    environment = terraform.workspace
    version     = var.image_tag
  }
}
