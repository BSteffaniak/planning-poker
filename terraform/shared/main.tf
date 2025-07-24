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
  }
}

provider "aws" {
  region = "us-east-1"
}

provider "cloudflare" {
  # API token will be read from CLOUDFLARE_API_TOKEN environment variable
}

# Data source for Cloudflare zone
data "cloudflare_zone" "main" {
  name = "hyperchad.dev"
}
