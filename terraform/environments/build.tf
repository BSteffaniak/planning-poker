# Build static assets
resource "terraform_data" "build_static" {
  triggers_replace = {
    environment          = terraform.workspace
    cargo_toml           = filemd5("${path.module}/../../Cargo.toml")
    packages_cargo_tomls = md5(join("", [for f in fileset("${path.module}/../../packages", "**/*.toml") : filemd5("${path.module}/../../packages/${f}")]))
    packages_src_files   = md5(join("", [for f in fileset("${path.module}/../../packages", "**/*.rs") : filemd5("${path.module}/../../packages/${f}")]))
  }

  provisioner "local-exec" {
    command = "bash ${path.module}/../scripts/build-static.sh"
    environment = {
      ENVIRONMENT = terraform.workspace
      OUTPUT_DIR  = "${path.module}/../../packages/app/gen"
    }
    working_dir = path.module
  }
}

# Build Lambda function
resource "terraform_data" "build_lambda" {
  triggers_replace = {
    environment          = terraform.workspace
    debug_mode           = var.debug_mode
    cargo_toml           = filemd5("${path.module}/../../Cargo.toml")
    packages_cargo_tomls = md5(join("", [for f in fileset("${path.module}/../../packages", "**/*.toml") : filemd5("${path.module}/../../packages/${f}")]))
    packages_src_files   = md5(join("", [for f in fileset("${path.module}/../../packages", "**/*.rs") : filemd5("${path.module}/../../packages/${f}")]))
  }

  provisioner "local-exec" {
    command = "bash ${path.module}/../scripts/build-lambda.sh"
    environment = {
      ENVIRONMENT = terraform.workspace
      DEBUG_MODE  = var.debug_mode ? "true" : "false"
    }
    working_dir = path.module
  }
}

# Upload static assets to S3
resource "terraform_data" "upload_assets" {
  depends_on = [
    terraform_data.build_static,
    aws_s3_bucket.assets,
    aws_s3_bucket_policy.assets
  ]

  triggers_replace = {
    environment          = terraform.workspace
    debug_mode           = var.debug_mode
    cargo_toml           = filemd5("${path.module}/../../Cargo.toml")
    packages_cargo_tomls = md5(join("", [for f in fileset("${path.module}/../../packages", "**/*.toml") : filemd5("${path.module}/../../packages/${f}")]))
    packages_src_files   = md5(join("", [for f in fileset("${path.module}/../../packages", "**/*.rs") : filemd5("${path.module}/../../packages/${f}")]))
    build_static_id      = terraform_data.build_static.id
  }

  provisioner "local-exec" {
    command = "bash ${path.module}/../scripts/upload-assets.sh"
    environment = {
      BUCKET_NAME = aws_s3_bucket.assets.bucket
      SOURCE_DIR  = "${path.module}/../../packages/app/gen"
    }
    working_dir = path.module
  }
}
