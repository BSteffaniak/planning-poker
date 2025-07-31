
# DigitalOcean Configuration
variable "digitalocean_region" {
  description = "DigitalOcean region for resources"
  type        = string
  default     = "nyc1"
}

variable "digitalocean_node_size" {
  description = "Size of Kubernetes nodes"
  type        = string
  default     = "s-1vcpu-2gb"
}

variable "digitalocean_node_count" {
  description = "Number of Kubernetes nodes"
  type        = number
  default     = 1
}

variable "digitalocean_auto_scale" {
  description = "Enable auto-scaling for node pool"
  type        = bool
  default     = false  # Disable auto-scaling for single node setup
}

variable "digitalocean_min_nodes" {
  description = "Minimum number of nodes for auto-scaling"
  type        = number
  default     = 1
}

variable "digitalocean_max_nodes" {
  description = "Maximum number of nodes for auto-scaling"
  type        = number
  default     = 1  # Restrict to single node maximum
}

variable "container_registry_tier" {
  description = "Container registry subscription tier"
  type        = string
  default     = "basic"
}

# Application Configuration
variable "image_tag" {
  description = "Container image tag to deploy"
  type        = string
  default     = "latest"
}

variable "database_url" {
  description = "Database connection URL (PostgreSQL) - uses DATABASE_URL env var if not provided"
  type        = string
  sensitive   = true
  default     = null
}

variable "enable_debug_logging" {
  description = "Enable debug logging for the application"
  type        = bool
  default     = false
}

variable "enable_trace_logging" {
  description = "Enable trace logging for the application"
  type        = bool
  default     = false
}

# Kubernetes Configuration
variable "k8s_replicas" {
  description = "Number of application replicas"
  type        = number
  default     = 1
}

variable "k8s_environment_variables" {
  description = "Additional environment variables for the application"
  type        = map(string)
  default     = {}
}

variable "k8s_cpu_request" {
  description = "CPU request for application pods"
  type        = string
  default     = "100m"
}

variable "k8s_memory_request" {
  description = "Memory request for application pods"
  type        = string
  default     = "128Mi"
}

variable "k8s_cpu_limit" {
  description = "CPU limit for application pods"
  type        = string
  default     = "500m"
}

variable "k8s_memory_limit" {
  description = "Memory limit for application pods"
  type        = string
  default     = "512Mi"
}

# SSL/TLS Configuration
variable "cert_manager_issuer" {
  description = "Cert-manager cluster issuer name"
  type        = string
  default     = "letsencrypt-prod"
}

variable "letsencrypt_email" {
  description = "Email address for Let's Encrypt certificate notifications"
  type        = string
  default     = "BradenSteffaniak@gmail.com"
}

variable "use_ssl" {
  description = "Enable SSL/TLS with automatic certificate provisioning via DNS-01"
  type        = bool
  default     = true
}

variable "cloudflare_api_token" {
  description = "Cloudflare API token for DNS-01 challenges"
  type        = string
  sensitive   = true
}

# Horizontal Pod Autoscaler
variable "enable_hpa" {
  description = "Enable Horizontal Pod Autoscaler"
  type        = bool
  default     = false
}

variable "hpa_min_replicas" {
  description = "Minimum replicas for HPA"
  type        = number
  default     = 1
}

variable "hpa_max_replicas" {
  description = "Maximum replicas for HPA"
  type        = number
  default     = 5
}

variable "hpa_cpu_target" {
  description = "Target CPU utilization percentage for HPA"
  type        = number
  default     = 70
}

variable "hpa_memory_target" {
  description = "Target memory utilization percentage for HPA"
  type        = number
  default     = 80
}
