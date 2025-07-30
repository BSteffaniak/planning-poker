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
  }
}

provider "digitalocean" {
  # Token will be read from DIGITALOCEAN_TOKEN environment variable
}

provider "cloudflare" {
  # API token will be read from CLOUDFLARE_API_TOKEN environment variable
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

  common_tags = {
    Environment = terraform.workspace
    Project     = "planning-poker"
    ManagedBy   = "terraform"
  }
}
