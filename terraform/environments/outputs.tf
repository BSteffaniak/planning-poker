output "website_url" {
  description = "URL of the deployed website"
  value       = "https://${local.subdomain}"
}

output "kubernetes_cluster_id" {
  description = "DigitalOcean Kubernetes cluster ID"
  value       = digitalocean_kubernetes_cluster.planning_poker.id
}

output "kubernetes_cluster_endpoint" {
  description = "Kubernetes cluster endpoint"
  value       = digitalocean_kubernetes_cluster.planning_poker.endpoint
  sensitive   = true
}

output "container_registry_endpoint" {
  description = "Container registry endpoint"
  value       = digitalocean_container_registry.planning_poker.endpoint
}

output "kubernetes_namespace" {
  description = "Kubernetes namespace for the application"
  value       = kubernetes_namespace.planning_poker.metadata[0].name
}

output "application_service_name" {
  description = "Kubernetes service name for the application"
  value       = kubernetes_service.planning_poker.metadata[0].name
}

output "node_external_ip" {
  description = "External IP of the Kubernetes node (where DNS points)"
  value       = local.first_node_external_ip
}

output "service_type" {
  description = "Service type used (ClusterIP with hostNetwork)"
  value       = "ClusterIP"
}

output "host_port" {
  description = "Host port bound directly via hostNetwork"
  value       = 80
}

output "cloudflare_proxy_enabled" {
  description = "Whether Cloudflare proxy is enabled for SSL termination"
  value       = true
}

output "direct_access_url" {
  description = "Direct access URL (bypasses Cloudflare)"
  value       = "http://${local.first_node_external_ip}:80"
}

output "r2_bucket_name" {
  description = "Cloudflare R2 bucket name for static assets"
  value       = cloudflare_r2_bucket.static_assets.name
}

output "r2_bucket_endpoint" {
  description = "Cloudflare R2 bucket endpoint for uploads"
  value       = "https://${cloudflare_r2_bucket.static_assets.name}.r2.cloudflarestorage.com"
}

output "static_asset_paths" {
  description = "Paths that route to R2 bucket"
  value = [
    "/public/*",
    "/js/*",
    "/favicon.ico"
  ]
}
