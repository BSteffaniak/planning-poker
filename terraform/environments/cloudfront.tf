# CloudFront Origin Access Control for S3
resource "aws_cloudfront_origin_access_control" "s3" {
  name                              = "planning-poker-${terraform.workspace}-s3-oac"
  description                       = "OAC for Planning Poker S3 bucket"
  origin_access_control_origin_type = "s3"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

# Cache policy for streaming responses
resource "aws_cloudfront_cache_policy" "streaming" {
  name        = "planning-poker-streaming-${terraform.workspace}"
  comment     = "Cache policy for streaming SSE responses"
  default_ttl = 0
  max_ttl     = 0
  min_ttl     = 0

  parameters_in_cache_key_and_forwarded_to_origin {
    enable_accept_encoding_brotli = false
    enable_accept_encoding_gzip   = false

    query_strings_config {
      query_string_behavior = "none"
    }

    headers_config {
      header_behavior = "none"
    }

    cookies_config {
      cookie_behavior = "none"
    }
  }
}

# Origin request policy for streaming
resource "aws_cloudfront_origin_request_policy" "streaming" {
  name    = "planning-poker-streaming-origin-${terraform.workspace}"
  comment = "Origin request policy for streaming responses"

  query_strings_config {
    query_string_behavior = "all"
  }

  headers_config {
    header_behavior = "whitelist"
    headers {
      items = [
        "CloudFront-Forwarded-Proto",
        "Cache-Control",
        "Accept"
      ]
    }
  }

  cookies_config {
    cookie_behavior = "all"
  }
}

# CloudFront distribution
resource "aws_cloudfront_distribution" "main" {
  depends_on = [
    aws_lambda_function_url.app,
    terraform_data.build_lambda
  ]

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
    origin_id   = "Lambda-${local.lambda_function_name}"

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }

  enabled             = true
  is_ipv6_enabled     = true
  default_root_object = "index.html"

  tags = local.common_tags

  # Default behavior - serve static assets from S3 (including index.html)
  default_cache_behavior {
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

  # SSE endpoint - separate behavior with no caching
  ordered_cache_behavior {
    path_pattern           = "/$sse"
    allowed_methods        = ["GET", "HEAD", "OPTIONS"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "Lambda-${local.lambda_function_name}"
    compress               = false  # Don't compress SSE streams
    viewer_protocol_policy = "redirect-to-https"

    # Use managed policies for streaming
    cache_policy_id          = aws_cloudfront_cache_policy.streaming.id
    origin_request_policy_id = aws_cloudfront_origin_request_policy.streaming.id

    min_ttl     = 0
    default_ttl = 0
    max_ttl     = 0
  }

  # Dynamic routes go to Lambda
  dynamic "ordered_cache_behavior" {
    for_each = ["/api/*", "/game/*", "/join-game", "/__hyperchad_dynamic_root__"]

    content {
      path_pattern           = ordered_cache_behavior.value
      allowed_methods        = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
      cached_methods         = ["GET", "HEAD"]
      target_origin_id       = "Lambda-${local.lambda_function_name}"
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


