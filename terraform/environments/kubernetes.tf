# Kubernetes namespace
resource "kubernetes_namespace" "planning_poker" {
  metadata {
    name = local.k8s_namespace
    labels = local.k8s_labels
  }
}

# Secret for database connection
resource "kubernetes_secret" "planning_poker_secrets" {
  metadata {
    name      = "planning-poker-secrets"
    namespace = kubernetes_namespace.planning_poker.metadata[0].name
    labels    = local.k8s_labels
  }

  data = {
    database-url = var.database_url != null ? var.database_url : env("DATABASE_URL")
  }

  type = "Opaque"
}

# Secret for container registry access
resource "kubernetes_secret" "registry_credentials" {
  metadata {
    name      = "registry-credentials"
    namespace = kubernetes_namespace.planning_poker.metadata[0].name
    labels    = local.k8s_labels
  }

  data = {
    ".dockerconfigjson" = digitalocean_container_registry_docker_credentials.planning_poker.docker_credentials
  }

  type = "kubernetes.io/dockerconfigjson"
}

# ConfigMap for application configuration
resource "kubernetes_config_map" "planning_poker_config" {
  metadata {
    name      = "planning-poker-config"
    namespace = kubernetes_namespace.planning_poker.metadata[0].name
    labels    = local.k8s_labels
  }

  data = {
    ENVIRONMENT = terraform.workspace
    RUST_LOG    = var.enable_trace_logging ? "planning_poker=trace,hyperchad=trace" : var.enable_debug_logging ? "planning_poker=debug,hyperchad=debug" : "planning_poker=info"
    PORT        = "80"  # App must listen on port 80 with hostNetwork
  }
}

# Deployment
resource "kubernetes_deployment" "planning_poker" {
  metadata {
    name      = local.app_name
    namespace = kubernetes_namespace.planning_poker.metadata[0].name
    labels    = local.k8s_labels
  }

  spec {
    replicas = var.k8s_replicas

    strategy {
      type = "Recreate"
    }

    selector {
      match_labels = {
        app = local.app_name
      }
    }

    template {
      metadata {
        labels = local.k8s_labels
      }

      spec {
        host_network = true  # Enable host networking for direct port binding

        image_pull_secrets {
          name = kubernetes_secret.registry_credentials.metadata[0].name
        }

        container {
          name              = local.app_name
          image             = local.container_image
          image_pull_policy = "Always"

          port {
            container_port = 80  # Must match host_port with hostNetwork
            host_port     = 80  # External port for Cloudflare
            protocol       = "TCP"
          }

          env_from {
            config_map_ref {
              name = kubernetes_config_map.planning_poker_config.metadata[0].name
            }
          }

          env {
            name = "DATABASE_URL"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.planning_poker_secrets.metadata[0].name
                key  = "database-url"
              }
            }
          }

          # Add any additional environment variables
          dynamic "env" {
            for_each = var.k8s_environment_variables
            content {
              name  = env.key
              value = env.value
            }
          }

          resources {
            requests = {
              cpu    = var.k8s_cpu_request
              memory = var.k8s_memory_request
            }
            limits = {
              cpu    = var.k8s_cpu_limit
              memory = var.k8s_memory_limit
            }
          }

          liveness_probe {
            http_get {
              path = "/health"
              port = 80  # App now listens on port 80
            }
            initial_delay_seconds = 30
            period_seconds        = 10
            timeout_seconds       = 5
            failure_threshold     = 3
          }

          readiness_probe {
            http_get {
              path = "/health"
              port = 80  # App now listens on port 80
            }
            initial_delay_seconds = 5
            period_seconds        = 5
            timeout_seconds       = 3
            failure_threshold     = 3
          }
        }

        restart_policy = "Always"
      }
    }
  }

  depends_on = [
    kubernetes_secret.planning_poker_secrets,
    kubernetes_secret.registry_credentials,
    kubernetes_config_map.planning_poker_config
  ]
}

# Service - ClusterIP (hostNetwork handles external access directly)
resource "kubernetes_service" "planning_poker" {
  metadata {
    name      = "planning-poker-service"
    namespace = kubernetes_namespace.planning_poker.metadata[0].name
    labels    = local.k8s_labels
  }

  spec {
    selector = {
      app = local.app_name
    }

    port {
      name        = "http"
      port        = 80
      target_port = 80  # App now listens on port 80
      protocol    = "TCP"
    }

    type = "ClusterIP"
  }
}







# Horizontal Pod Autoscaler (optional)
resource "kubernetes_horizontal_pod_autoscaler_v2" "planning_poker" {
  count = var.enable_hpa ? 1 : 0

  metadata {
    name      = "planning-poker-hpa"
    namespace = kubernetes_namespace.planning_poker.metadata[0].name
    labels    = local.k8s_labels
  }

  spec {
    scale_target_ref {
      api_version = "apps/v1"
      kind        = "Deployment"
      name        = kubernetes_deployment.planning_poker.metadata[0].name
    }

    min_replicas = var.hpa_min_replicas
    max_replicas = var.hpa_max_replicas

    metric {
      type = "Resource"
      resource {
        name = "cpu"
        target {
          type                = "Utilization"
          average_utilization = var.hpa_cpu_target
        }
      }
    }

    metric {
      type = "Resource"
      resource {
        name = "memory"
        target {
          type                = "Utilization"
          average_utilization = var.hpa_memory_target
        }
      }
    }
  }

  depends_on = [kubernetes_deployment.planning_poker]
}
