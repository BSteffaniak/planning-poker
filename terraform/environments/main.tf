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
  environment = terraform.workspace
  is_prod     = terraform.workspace == "prod"
  subdomain   = local.is_prod ? "planning-poker.hyperchad.dev" : "${terraform.workspace}.planning-poker.hyperchad.dev"
  bucket_name = "planning-poker-${terraform.workspace}-${random_id.suffix.hex}"

  # Lambda function name abstraction for debug/release modes
  lambda_function_name = var.debug_mode ? aws_lambda_function.app_debug[0].function_name : aws_lambda_function.app_release[0].function_name

  common_tags = {
    Environment = terraform.workspace
    Project     = "planning-poker"
    ManagedBy   = "terraform"
  }
}
