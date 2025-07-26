
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

variable "enable_trace_logging" {
  description = "Enable trace logging for the Lambda function"
  type        = bool
  default     = false
}

variable "debug_mode" {
  description = "Enable debug mode (debug build, full logging, backtraces)"
  type        = bool
  default     = false
}

variable "database_url" {
  description = "Database connection URL (PostgreSQL)"
  type        = string
  sensitive   = true
  default     = null
}
