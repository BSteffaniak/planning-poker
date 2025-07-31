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
    PORT        = tostring(local.app_port)
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
        image_pull_secrets {
          name = kubernetes_secret.registry_credentials.metadata[0].name
        }

        container {
          name  = local.app_name
          image = local.container_image

          port {
            container_port = local.app_port
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
              port = local.app_port
            }
            initial_delay_seconds = 30
            period_seconds        = 10
            timeout_seconds       = 5
            failure_threshold     = 3
          }

          readiness_probe {
            http_get {
              path = "/health"
              port = local.app_port
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

# Service
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
      target_port = local.app_port
      protocol    = "TCP"
    }

    type = "ClusterIP"
  }
}

# Wildcard Certificate for SSL/TLS using kubectl provider
resource "kubectl_manifest" "planning_poker_certificate" {
  yaml_body = yamlencode({
    apiVersion = "cert-manager.io/v1"
    kind       = "Certificate"
    metadata = {
      name      = "planning-poker-wildcard-tls"
      namespace = kubernetes_namespace.planning_poker.metadata[0].name
    }
    spec = {
      secretName = "planning-poker-tls"
      issuerRef = {
        name = var.cert_manager_issuer
        kind = "ClusterIssuer"
      }
      dnsNames = [
        "*.planning-poker.hyperchad.dev",  # Wildcard covers all subdomains
        "planning-poker.hyperchad.dev"     # Also cover root domain
      ]
      duration    = "2160h"  # 90 days
      renewBefore = "360h"   # 15 days before expiry
      privateKey = {
        algorithm = "RSA"
        size      = 2048
      }
    }
  })

  depends_on = [
    kubernetes_namespace.planning_poker,
    kubectl_manifest.letsencrypt_prod
  ]
}

# MoosicBox Load Balancer Deployment
resource "kubernetes_deployment" "moosicbox_lb" {
  metadata {
    name      = "moosicbox-lb"
    namespace = kubernetes_namespace.planning_poker.metadata[0].name
    labels    = merge(local.k8s_labels, { component = "load-balancer" })
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        app = "moosicbox-lb"
      }
    }

    template {
      metadata {
        labels = merge(local.k8s_labels, {
          app       = "moosicbox-lb"
          component = "load-balancer"
        })
      }

      spec {
        image_pull_secrets {
          name = kubernetes_secret.registry_credentials.metadata[0].name
        }

        container {
          name  = "moosicbox-lb"
          image = "${digitalocean_container_registry.planning_poker.endpoint}/moosicbox-lb:${var.image_tag}"

          port {
            container_port = 80
            host_port     = 80  # Bind directly to node port 80
            protocol       = "TCP"
            name          = "http"
          }
          port {
            container_port = 443
            host_port     = 443  # Bind directly to node port 443
            protocol       = "TCP"
            name          = "https"
          }

          env {
            name  = "CLUSTERS"
            value = "${local.subdomain}:${kubernetes_service.planning_poker.metadata[0].name}.${kubernetes_namespace.planning_poker.metadata[0].name}.svc.cluster.local:80;solver:${kubernetes_service.acme_solver.metadata[0].name}.${kubernetes_namespace.planning_poker.metadata[0].name}.svc.cluster.local:80"
          }
          env {
            name  = "PORT"
            value = "80"  # Use standard HTTP port
          }
          env {
            name  = "SSL_PORT"
            value = "443"  # Use standard HTTPS port
          }
          env {
            name  = "SSL_CRT_PATH"
            value = "/etc/ssl/certs/tls.crt"
          }
          env {
            name  = "SSL_KEY_PATH"
            value = "/etc/ssl/private/tls.key"
          }

          volume_mount {
            name       = "tls-certs"
            mount_path = "/etc/ssl/certs"
            read_only  = true
          }
          volume_mount {
            name       = "tls-private"
            mount_path = "/etc/ssl/private"
            read_only  = true
          }

          resources {
            requests = {
              cpu    = "100m"
              memory = "128Mi"
            }
            limits = {
              cpu    = "200m"
              memory = "256Mi"
            }
          }

          liveness_probe {
            tcp_socket {
              port = 80  # Update to use standard HTTP port
            }
            initial_delay_seconds = 30
            period_seconds        = 10
            timeout_seconds       = 5
            failure_threshold     = 3
          }

          readiness_probe {
            tcp_socket {
              port = 80  # Update to use standard HTTP port
            }
            initial_delay_seconds = 5
            period_seconds        = 5
            timeout_seconds       = 3
            failure_threshold     = 3
          }
        }

        volume {
          name = "tls-certs"
          secret {
            secret_name = "planning-poker-tls"
            items {
              key  = "tls.crt"
              path = "tls.crt"
            }
          }
        }
        volume {
          name = "tls-private"
          secret {
            secret_name = "planning-poker-tls"
            items {
              key  = "tls.key"
              path = "tls.key"
            }
          }
        }
      }
    }
  }

  depends_on = [
    kubernetes_secret.registry_credentials,
    kubectl_manifest.planning_poker_certificate
  ]
}

# MoosicBox Load Balancer Service (NodePort to avoid LoadBalancer costs)
resource "kubernetes_service" "moosicbox_lb" {
  metadata {
    name      = "moosicbox-lb-service"
    namespace = kubernetes_namespace.planning_poker.metadata[0].name
    labels    = merge(local.k8s_labels, { component = "load-balancer" })

    annotations = {
      "service.beta.kubernetes.io/do-loadbalancer-enable-proxy-protocol" = "true"
      "service.beta.kubernetes.io/do-loadbalancer-hostname" = local.subdomain
    }
  }

  spec {
    type = "NodePort"  # Use NodePort instead of LoadBalancer to avoid costs
    ip_families = ["IPv4"]
    ip_family_policy = "SingleStack"

    selector = {
      app = "moosicbox-lb"
    }

    port {
      name        = "http"
      port        = 80
      target_port = "http"
      protocol    = "TCP"
    }
    port {
      name        = "https"
      port        = 443
      target_port = "https"
      protocol    = "TCP"
    }
  }

  depends_on = [kubernetes_deployment.moosicbox_lb]
}

# Simple HTTP service for ACME challenges
resource "kubernetes_deployment" "acme_solver" {
  metadata {
    name      = "acme-solver"
    namespace = kubernetes_namespace.planning_poker.metadata[0].name
    labels    = merge(local.k8s_labels, { component = "acme-solver" })
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        app = "acme-solver"
      }
    }

    template {
      metadata {
        labels = merge(local.k8s_labels, {
          app       = "acme-solver"
          component = "acme-solver"
        })
      }

      spec {
        container {
          name  = "acme-solver"
          image = "nginx:alpine"

          port {
            container_port = 80
            protocol       = "TCP"
          }

          resources {
            requests = {
              cpu    = "10m"
              memory = "16Mi"
            }
            limits = {
              cpu    = "50m"
              memory = "32Mi"
            }
          }
        }
      }
    }
  }
}

resource "kubernetes_service" "acme_solver" {
  metadata {
    name      = "acme-solver-service"
    namespace = kubernetes_namespace.planning_poker.metadata[0].name
    labels    = merge(local.k8s_labels, { component = "acme-solver" })
  }

  spec {
    type = "ClusterIP"
    selector = {
      app = "acme-solver"
    }

    port {
      name        = "http"
      port        = 80
      target_port = 80
      protocol    = "TCP"
    }
  }

  depends_on = [kubernetes_deployment.acme_solver]
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
