terraform {
  required_version = ">= 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
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

provider "aws" {
  region = "us-east-1"
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
  environment = var.environment
  subdomain   = var.environment == "prod" ? "planning-poker.hyperchad.dev" : "${var.environment}.planning-poker.hyperchad.dev"
  bucket_name = "planning-poker-${var.environment}-${random_id.suffix.hex}"

  common_tags = {
    Environment = var.environment
    Project     = "planning-poker"
    ManagedBy   = "terraform"
  }
}
