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

output "ingress_hostname" {
  description = "Ingress hostname for external access"
  value       = local.subdomain
}
