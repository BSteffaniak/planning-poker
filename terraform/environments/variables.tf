variable "environment" {
  description = "Environment name (dev, staging, prod, or feature branch name)"
  type        = string

  validation {
    condition     = can(regex("^[a-z0-9-]+$", var.environment))
    error_message = "Environment must contain only lowercase letters, numbers, and hyphens."
  }
}

variable "lambda_environment_variables" {
  description = "Environment variables for the Lambda function"
  type        = map(string)
  default     = {}
}

variable "enable_debug_logging" {
  description = "Enable debug logging for the Lambda function"
  type        = bool
  default     = false
}

variable "debug_mode" {
  description = "Enable debug mode (debug build, full logging, backtraces)"
  type        = bool
  default     = false
}
