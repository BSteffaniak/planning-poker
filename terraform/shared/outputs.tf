output "certificate_arn" {
  description = "ARN of the planning poker SSL certificate"
  value       = aws_acm_certificate_validation.planning_poker.certificate_arn
}

output "cloudflare_zone_id" {
  description = "Cloudflare zone ID for hyperchad.dev"
  value       = data.cloudflare_zone.main.id
}

output "cloudflare_zone_name" {
  description = "Cloudflare zone name"
  value       = data.cloudflare_zone.main.name
}
