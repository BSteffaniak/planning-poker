# Planning Poker SSL certificate (covers both root and wildcard)
resource "aws_acm_certificate" "planning_poker" {
  domain_name               = "planning-poker.hyperchad.dev"
  subject_alternative_names = ["*.planning-poker.hyperchad.dev"]
  validation_method         = "DNS"

  lifecycle {
    create_before_destroy = true
  }

  tags = {
    Name        = "Planning Poker Certificate"
    Environment = "shared"
    Project     = "planning-poker"
  }
}

# Local value for certificate validation record
locals {
  # Get the first (and effectively only unique) validation record
  cert_validation = tolist(aws_acm_certificate.planning_poker.domain_validation_options)[0]
}

# DNS validation record for the planning poker certificate via Cloudflare
resource "cloudflare_record" "planning_poker_cert_validation" {
  zone_id = data.cloudflare_zone.main.id
  name    = local.cert_validation.resource_record_name
  content = local.cert_validation.resource_record_value
  type    = local.cert_validation.resource_record_type
  ttl     = 60
}

# Planning poker certificate validation
resource "aws_acm_certificate_validation" "planning_poker" {
  certificate_arn         = aws_acm_certificate.planning_poker.arn
  validation_record_fqdns = [cloudflare_record.planning_poker_cert_validation.hostname]

  timeouts {
    create = "10m"
  }
}
