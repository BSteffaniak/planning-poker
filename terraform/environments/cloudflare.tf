# Cloudflare DNS record for the application
resource "cloudflare_record" "main" {
  zone_id = data.cloudflare_zone.main.id
  name    = local.subdomain_name
  content = try(kubernetes_ingress_v1.planning_poker.status[0].load_balancer[0].ingress[0].ip, "0.0.0.0")
  type    = "A"
  proxied = false  # DNS-only, no CDN proxy - direct to LoadBalancer
  ttl     = 300    # 5 minutes

  comment = "Planning Poker ${terraform.workspace} environment - points to Kubernetes LoadBalancer"

  depends_on = [kubernetes_ingress_v1.planning_poker]

  lifecycle {
    ignore_changes = [content]  # Don't recreate if LoadBalancer IP changes
  }
}

# Local value for subdomain name (without the base domain)
locals {
  subdomain_name = terraform.workspace == "prod" ? "planning-poker" : "${terraform.workspace}.planning-poker"
}
