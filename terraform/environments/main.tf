terraform {
  required_version = ">= 1.0"

  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.0"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.0"
    }
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 4.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.1"
    }
    null = {
      source  = "hashicorp/null"
      version = "~> 3.1"
    }
    kubectl = {
      source  = "gavinbunney/kubectl"
      version = ">= 1.7.0"
    }
    time = {
      source  = "hashicorp/time"
      version = "~> 0.9"
    }
  }
}

provider "digitalocean" {
  # Token will be read from DIGITALOCEAN_TOKEN environment variable
}

provider "cloudflare" {
  # API token will be read from CLOUDFLARE_API_TOKEN environment variable
}

provider "kubectl" {
  host  = digitalocean_kubernetes_cluster.planning_poker.endpoint
  token = digitalocean_kubernetes_cluster.planning_poker.kube_config[0].token
  cluster_ca_certificate = base64decode(
    digitalocean_kubernetes_cluster.planning_poker.kube_config[0].cluster_ca_certificate
  )
}

# Random suffix for unique resource names
resource "random_id" "suffix" {
  byte_length = 4
}

# Data sources for shared resources
data "terraform_remote_state" "shared" {
  backend = "local"
  config = {
    path = "../shared/terraform.tfstate"
  }
}

data "cloudflare_zone" "main" {
  name = "hyperchad.dev"
}

# Local values
locals {
  environment = terraform.workspace
  is_prod     = terraform.workspace == "prod"
  subdomain   = local.is_prod ? "planning-poker.hyperchad.dev" : "${terraform.workspace}.planning-poker.hyperchad.dev"

  # Cloudflare API token from TF_VAR_cloudflare_api_token environment variable
  cloudflare_api_token = var.cloudflare_api_token

  common_tags = {
    Environment = terraform.workspace
    Project     = "planning-poker"
    ManagedBy   = "terraform"
  }
}
