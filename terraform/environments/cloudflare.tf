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

# Cloudflare DNS record for the application - points directly to node IP with proxy enabled
resource "cloudflare_dns_record" "main" {
  zone_id = data.cloudflare_zone.main.zone_id
  name    = local.subdomain_name
  content = local.first_node_external_ip
  type    = "A"
  proxied = true   # Enable Cloudflare proxy for SSL termination and CDN
  ttl     = 1      # Auto TTL when proxied

  comment = "Planning Poker ${terraform.workspace} environment - Cloudflare proxy enabled for SSL termination"

  depends_on = [
    kubernetes_service.planning_poker,
    data.kubernetes_nodes.cluster_nodes
  ]

  lifecycle {
    ignore_changes = [content]  # Don't recreate if node IP changes
  }
}

# Cloudflare zone settings - commented out to avoid read-only property errors
# Cloudflare defaults will work fine for basic SSL functionality
# resource "cloudflare_zone_settings_override" "main" {
#   zone_id = data.cloudflare_zone.main.zone_id
#
#   settings {
#     ssl                      = "flexible"  # Cloudflare→Your server uses HTTP
#     always_use_https        = "on"
#     automatic_https_rewrites = "on"
#     min_tls_version         = "1.2"
#   }
# }



# Cloudflare R2 bucket for static assets (private)
resource "cloudflare_r2_bucket" "static_assets" {
  account_id = local.account_id
  name       = "planning-poker-assets-${terraform.workspace}"
  location   = "ENAM"  # Eastern North America
}



# Connect R2 bucket to custom domain and enable public access
resource "cloudflare_r2_custom_domain" "static_assets_domain" {
  account_id  = local.account_id
  bucket_name = cloudflare_r2_bucket.static_assets.name
  domain      = "planning-poker-assets-${terraform.workspace}.hyperchad.dev"
  enabled     = true
  zone_id     = data.cloudflare_zone.main.zone_id

  depends_on = [
    cloudflare_r2_bucket.static_assets
  ]
}

# Single Redirects using cloudflare_ruleset for http_request_dynamic_redirect phase
resource "cloudflare_ruleset" "redirect_rules" {
  zone_id = data.cloudflare_zone.main.zone_id
  name    = "Static Asset Redirects"
  kind    = "zone"
  phase   = "http_request_dynamic_redirect"

  rules = [
    {
      ref         = "redirect_public_to_r2"
      expression  = "starts_with(http.request.uri.path, \"/public/\") and http.host eq \"${local.subdomain_name}.hyperchad.dev\""
      description = "Redirect /public/* to R2 bucket"
      action      = "redirect"

      action_parameters = {
        from_value = {
          target_url = {
            expression = "concat(\"https://planning-poker-assets-${terraform.workspace}.hyperchad.dev\", http.request.uri.path)"
          }
          status_code = 301
        }
      }
    },
    {
      ref         = "redirect_js_to_r2"
      expression  = "starts_with(http.request.uri.path, \"/js/\") and http.host eq \"${local.subdomain_name}.hyperchad.dev\""
      description = "Redirect /js/* to R2 bucket"
      action      = "redirect"

      action_parameters = {
        from_value = {
          target_url = {
            expression = "concat(\"https://planning-poker-assets-${terraform.workspace}.hyperchad.dev\", http.request.uri.path)"
          }
          status_code = 301
        }
      }
    },
    {
      ref         = "redirect_favicon_to_r2"
      expression  = "http.request.uri.path eq \"/favicon.ico\" and http.host eq \"${local.subdomain_name}.hyperchad.dev\""
      description = "Redirect favicon.ico to R2 bucket"
      action      = "redirect"

      action_parameters = {
        from_value = {
          target_url = {
            value = "https://planning-poker-assets-${terraform.workspace}.hyperchad.dev/favicon.ico"
          }
          status_code = 301
        }
      }
    }
  ]

  depends_on = [cloudflare_r2_custom_domain.static_assets_domain]
}

# Local value for subdomain name (without the base domain)
locals {
  subdomain_name = terraform.workspace == "prod" ? "planning-poker" : "planning-poker-${terraform.workspace}"
}
