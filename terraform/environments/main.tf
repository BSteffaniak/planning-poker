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
      version = "~> 5.0"
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

provider "digitalocean" {  # Token will be read from DIGITALOCEAN_TOKEN environment variable
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
  filter = {
    name = "hyperchad.dev"
  }
}

# Hash the source code to detect changes
data "archive_file" "source_hash" {
  type        = "zip"
  source_dir  = "../../packages"
  output_path = "/tmp/planning-poker-source-${terraform.workspace}.zip"
  excludes    = [
    "target",
    "node_modules",
    ".git",
    "*.log"
  ]
}



# Local values
locals {
  environment = terraform.workspace
  is_prod     = terraform.workspace == "prod"
  subdomain   = local.is_prod ? "planning-poker.hyperchad.dev" : "planning-poker-${terraform.workspace}.hyperchad.dev"

  # Cloudflare API token from TF_VAR_cloudflare_api_token environment variable
  cloudflare_api_token = var.cloudflare_api_token

  # Account ID from variable
  account_id = var.cloudflare_account_id

  # Source code hash for image tagging
  source_hash = substr(data.archive_file.source_hash.output_md5, 0, 8)
  image_tag   = var.image_tag != "latest" ? var.image_tag : "build-${local.source_hash}"

  common_tags = {
    Environment = terraform.workspace
    Project     = "planning-poker"
    ManagedBy   = "terraform"
  }
}

# Trigger container build and push when source code changes
resource "null_resource" "build_and_push" {
  triggers = {
    source_hash = data.archive_file.source_hash.output_md5
    image_tag   = local.image_tag
  }

  provisioner "local-exec" {
    command = "../scripts/build-and-deploy.sh ${local.image_tag} ${terraform.workspace}"
  }

  depends_on = [
    digitalocean_container_registry.planning_poker,
    digitalocean_kubernetes_cluster.planning_poker
  ]
}

# Trigger static asset upload when source code changes
resource "null_resource" "upload_assets" {
  triggers = {
    source_hash = data.archive_file.source_hash.output_md5
    bucket_name = cloudflare_r2_bucket.static_assets.name
  }

  provisioner "local-exec" {
    command = "../scripts/upload-assets.sh"
    environment = {
      SOURCE_DIR = "../../packages/app/gen/"
      BUCKET_NAME = cloudflare_r2_bucket.static_assets.name
      R2_ACCOUNT_ID = local.account_id
      AWS_ACCESS_KEY_ID = var.r2_access_key_id
      AWS_SECRET_ACCESS_KEY = var.r2_secret_access_key
    }
  }

  depends_on = [
    cloudflare_r2_custom_domain.static_assets_domain,
    null_resource.build_and_push
  ]
}
