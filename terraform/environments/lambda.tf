# S3 bucket for Lambda packages (debug mode only)
resource "aws_s3_bucket" "lambda_packages" {
  count  = var.debug_mode ? 1 : 0
  bucket = "planning-poker-lambda-${terraform.workspace}-${random_id.suffix.hex}"
  tags   = local.common_tags
}

resource "aws_s3_bucket_versioning" "lambda_packages" {
  count  = var.debug_mode ? 1 : 0
  bucket = aws_s3_bucket.lambda_packages[0].id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "lambda_packages" {
  count  = var.debug_mode ? 1 : 0
  bucket = aws_s3_bucket.lambda_packages[0].id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# IAM role for Lambda function
resource "aws_iam_role" "lambda" {
  name = "planning-poker-lambda-${terraform.workspace}"
  tags = local.common_tags

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })
}

# IAM policy attachment for Lambda basic execution
resource "aws_iam_role_policy_attachment" "lambda_basic" {
  role       = aws_iam_role.lambda.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

# IAM policy for Lambda to access S3 bucket (debug mode only)
resource "aws_iam_role_policy" "lambda_s3_access" {
  count = var.debug_mode ? 1 : 0
  name  = "lambda-s3-access"
  role  = aws_iam_role.lambda.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:GetObjectVersion"
        ]
        Resource = "${aws_s3_bucket.lambda_packages[0].arn}/*"
      }
    ]
  })
}

# Upload Lambda package to S3 (debug mode only)
resource "aws_s3_object" "lambda_package" {
  count  = var.debug_mode ? 1 : 0
  bucket = aws_s3_bucket.lambda_packages[0].bucket
  key    = "bootstrap.zip"
  source = "${path.module}/../../target/lambda/planning-poker-app-lambda/bootstrap.zip"
  tags   = local.common_tags

  depends_on = [terraform_data.build_lambda]

  lifecycle {
    replace_triggered_by = [terraform_data.build_lambda]
  }
}

# Lambda function - Release mode (direct upload)
resource "aws_lambda_function" "app_release" {
  count = var.debug_mode ? 0 : 1

  filename         = "${path.module}/../../target/lambda/planning-poker-app-lambda/bootstrap.zip"

  function_name    = "planning-poker-${terraform.workspace}-${formatdate("YYYYMMDD-hhmm", timestamp())}"
  role            = aws_iam_role.lambda.arn
  handler         = "bootstrap"
  runtime         = "provided.al2023"
  timeout         = 30
  memory_size     = 512

  tags = local.common_tags

  environment {
    variables = merge(
      {
        ENVIRONMENT = terraform.workspace
        RUST_LOG    = var.enable_trace_logging ? "planning_poker=trace,hyperchad=trace,moosicbox=trace,switchy=trace" : var.enable_debug_logging ? "planning_poker=debug,hyperchad=debug,moosicbox=debug,switchy=debug" : "planning_poker=info,moosicbox=info,switchy=info"
      },
      var.database_url != null ? { DATABASE_URL = var.database_url } : {},
      var.lambda_environment_variables
    )
  }

  lifecycle {
    create_before_destroy = true
    replace_triggered_by = [terraform_data.build_lambda]
  }

  depends_on = [terraform_data.build_lambda]
}

# Lambda function - Debug mode (S3 upload)
resource "aws_lambda_function" "app_debug" {
  count = var.debug_mode ? 1 : 0

  s3_bucket         = aws_s3_bucket.lambda_packages[0].bucket
  s3_key           = aws_s3_object.lambda_package[0].key
  s3_object_version = aws_s3_object.lambda_package[0].version_id

  function_name    = "planning-poker-${terraform.workspace}-${formatdate("YYYYMMDD-hhmm", timestamp())}"
  role            = aws_iam_role.lambda.arn
  handler         = "bootstrap"
  runtime         = "provided.al2023"
  timeout         = 30
  memory_size     = 512

  tags = local.common_tags

  environment {
    variables = merge(
      {
        ENVIRONMENT = terraform.workspace
        RUST_LOG    = "debug"
        RUST_BACKTRACE = "full"
      },
      var.database_url != null ? { DATABASE_URL = var.database_url } : {},
      var.lambda_environment_variables
    )
  }

  lifecycle {
    create_before_destroy = true
    replace_triggered_by = [
      terraform_data.build_lambda,
      aws_s3_object.lambda_package[0]
    ]
  }

  depends_on = [
    terraform_data.build_lambda,
    aws_s3_object.lambda_package[0]
  ]
}

# Lambda function URL (for API Gateway integration)
resource "aws_lambda_function_url" "app" {
  function_name      = var.debug_mode ? aws_lambda_function.app_debug[0].function_name : aws_lambda_function.app_release[0].function_name
  authorization_type = "NONE"
  invoke_mode        = "RESPONSE_STREAM"

  cors {
    allow_credentials = false
    allow_origins     = ["https://${local.subdomain}"]
    allow_methods     = ["GET", "POST", "PUT", "DELETE", "HEAD", "PATCH"]
    allow_headers     = ["date", "keep-alive", "content-type", "authorization", "cache-control", "accept"]
    expose_headers    = ["date", "keep-alive", "cache-control", "content-type"]
    max_age          = 86400
  }

  lifecycle {
    create_before_destroy = true
  }
}
