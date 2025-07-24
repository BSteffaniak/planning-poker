# Cloudflare DNS record for the application
resource "cloudflare_record" "main" {
  zone_id = data.cloudflare_zone.main.id
  name    = local.subdomain_name
  content = aws_cloudfront_distribution.main.domain_name
  type    = "CNAME"
  proxied = false  # DNS-only, no CDN proxy - direct to CloudFront
  ttl     = 300    # 5 minutes

  comment = "Planning Poker ${var.environment} environment - points to CloudFront"
}

# Local value for subdomain name (without the base domain)
locals {
  subdomain_name = var.environment == "prod" ? "planning-poker" : "${var.environment}.planning-poker"
}
