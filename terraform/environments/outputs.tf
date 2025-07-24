output "website_url" {
  description = "URL of the deployed website"
  value       = "https://${local.subdomain}"
}

output "cloudfront_distribution_id" {
  description = "CloudFront distribution ID"
  value       = aws_cloudfront_distribution.main.id
}

output "cloudfront_domain_name" {
  description = "CloudFront distribution domain name"
  value       = aws_cloudfront_distribution.main.domain_name
}

output "s3_bucket_name" {
  description = "S3 bucket name for static assets"
  value       = aws_s3_bucket.assets.bucket
}

output "lambda_function_name" {
  description = "Lambda function name"
  value       = var.debug_mode ? aws_lambda_function.app_debug[0].function_name : aws_lambda_function.app_release[0].function_name
}

output "lambda_function_url" {
  description = "Lambda function URL"
  value       = aws_lambda_function_url.app.function_url
}
