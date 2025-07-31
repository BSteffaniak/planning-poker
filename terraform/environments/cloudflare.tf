# Get node information from Kubernetes API (avoids DigitalOcean droplets permissions)
data "kubernetes_nodes" "cluster_nodes" {
  depends_on = [digitalocean_kubernetes_cluster.planning_poker]
}

# Extract external IP from first node
locals {
  first_node_external_ip = (
    length(data.kubernetes_nodes.cluster_nodes.nodes) > 0 ?
    try([for addr in data.kubernetes_nodes.cluster_nodes.nodes[0].status[0].addresses :
         addr.address if addr.type == "ExternalIP"][0], "0.0.0.0") :
    "0.0.0.0"
  )
}

# Cloudflare DNS record for the application - points directly to node IP
resource "cloudflare_record" "main" {
  zone_id = data.cloudflare_zone.main.id
  name    = local.subdomain_name
  content = local.first_node_external_ip
  type    = "A"
  proxied = false  # DNS-only, no CDN proxy - direct to node IP
  ttl     = 300    # 5 minutes

  comment = "Planning Poker ${terraform.workspace} environment - points directly to Kubernetes node IP (no LoadBalancer)"

  depends_on = [
    kubernetes_service.moosicbox_lb,
    data.kubernetes_nodes.cluster_nodes
  ]

  lifecycle {
    ignore_changes = [content]  # Don't recreate if node IP changes
  }
}

# Local value for subdomain name (without the base domain)
locals {
  subdomain_name = terraform.workspace == "prod" ? "planning-poker" : "${terraform.workspace}.planning-poker"
}
