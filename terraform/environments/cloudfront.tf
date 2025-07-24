# CloudFront Origin Access Control for S3
resource "aws_cloudfront_origin_access_control" "s3" {
  name                              = "planning-poker-${var.environment}-s3-oac"
  description                       = "OAC for Planning Poker S3 bucket"
  origin_access_control_origin_type = "s3"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

# CloudFront distribution
resource "aws_cloudfront_distribution" "main" {
  aliases = [local.subdomain]

  # S3 origin for static assets
  origin {
    domain_name              = aws_s3_bucket.assets.bucket_regional_domain_name
    origin_id                = "S3-${aws_s3_bucket.assets.id}"
    origin_access_control_id = aws_cloudfront_origin_access_control.s3.id
  }

  # Lambda origin for dynamic routes
  origin {
    domain_name = regex("https://([^/]+)", aws_lambda_function_url.app.function_url)[0]
    origin_id   = "Lambda-${var.debug_mode ? aws_lambda_function.app_debug[0].function_name : aws_lambda_function.app_release[0].function_name}"

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }

  enabled         = true
  is_ipv6_enabled = true

  tags = local.common_tags

  # Default behavior - serve dynamic content from Lambda
  default_cache_behavior {
    allowed_methods        = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "Lambda-${var.debug_mode ? aws_lambda_function.app_debug[0].function_name : aws_lambda_function.app_release[0].function_name}"
    compress               = true
    viewer_protocol_policy = "redirect-to-https"

    forwarded_values {
      query_string = true
      headers      = ["Authorization", "CloudFront-Forwarded-Proto"]
      cookies {
        forward = "all"
      }
    }

    min_ttl     = 0
    default_ttl = 0
    max_ttl     = 0
  }

  # Static assets behavior - serve from S3
  ordered_cache_behavior {
    path_pattern           = "/css/*"
    allowed_methods        = ["GET", "HEAD"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "S3-${aws_s3_bucket.assets.id}"
    compress               = true
    viewer_protocol_policy = "redirect-to-https"

    forwarded_values {
      query_string = false
      cookies {
        forward = "none"
      }
    }

    min_ttl     = 0
    default_ttl = 3600
    max_ttl     = 86400
  }

  ordered_cache_behavior {
    path_pattern           = "/img/*"
    allowed_methods        = ["GET", "HEAD"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "S3-${aws_s3_bucket.assets.id}"
    compress               = true
    viewer_protocol_policy = "redirect-to-https"

    forwarded_values {
      query_string = false
      cookies {
        forward = "none"
      }
    }

    min_ttl     = 0
    default_ttl = 3600
    max_ttl     = 86400
  }

  # Dynamic routes behavior - serve from Lambda
  dynamic "ordered_cache_behavior" {
    for_each = local.dynamic_routes

    content {
      path_pattern           = ordered_cache_behavior.value
      allowed_methods        = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
      cached_methods         = ["GET", "HEAD"]
      target_origin_id       = "Lambda-${var.debug_mode ? aws_lambda_function.app_debug[0].function_name : aws_lambda_function.app_release[0].function_name}"
      compress               = true
      viewer_protocol_policy = "redirect-to-https"

      forwarded_values {
        query_string = true
        headers      = ["Authorization", "CloudFront-Forwarded-Proto"]
        cookies {
          forward = "all"
        }
      }

      min_ttl     = 0
      default_ttl = 0
      max_ttl     = 0
    }
  }

  # SSL certificate configuration
  viewer_certificate {
    acm_certificate_arn      = data.terraform_remote_state.shared.outputs.certificate_arn
    ssl_support_method       = "sni-only"
    minimum_protocol_version = "TLSv1.2_2021"
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }
}

# DNS record is now managed in cloudflare.tf

# Local value for dynamic routes (will be populated by build process)
locals {
  dynamic_routes = [
    "/api/*",
    "/game/*",
    "/session/*"
  ]
}
