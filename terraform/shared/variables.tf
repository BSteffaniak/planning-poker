variable "cloudflare_api_token" {
  description = "Cloudflare API token for DNS management"
  type        = string
  sensitive   = true
  default     = null

  validation {
    condition     = var.cloudflare_api_token == null || can(regex("^[A-Za-z0-9_-]+$", var.cloudflare_api_token))
    error_message = "Cloudflare API token must be a valid token string."
  }
}
