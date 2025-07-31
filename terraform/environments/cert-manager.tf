# cert-manager installation and configuration with DNS-01 wildcard certificates

# cert-manager Helm chart
resource "helm_release" "cert_manager" {
  name       = "cert-manager"
  repository = "https://charts.jetstack.io"
  chart      = "cert-manager"
  version    = "v1.13.3"
  namespace  = "cert-manager"
  create_namespace = true

  values = [
    yamlencode({
      # Install CRDs automatically
      installCRDs = true

      # Global configuration
      global = {
        leaderElection = {
          namespace = "cert-manager"
        }
      }

      # Disable prometheus metrics
      prometheus = {
        enabled = false
      }

      # Resource limits for cert-manager controller
      resources = {
        requests = {
          cpu    = "10m"
          memory = "32Mi"
        }
        limits = {
          cpu    = "100m"
          memory = "128Mi"
        }
      }

      # Webhook resource limits
      webhook = {
        resources = {
          requests = {
            cpu    = "10m"
            memory = "32Mi"
          }
          limits = {
            cpu    = "100m"
            memory = "128Mi"
          }
        }
      }

      # CA Injector resource limits
      cainjector = {
        resources = {
          requests = {
            cpu    = "10m"
            memory = "32Mi"
          }
          limits = {
            cpu    = "100m"
            memory = "128Mi"
          }
        }
      }
    })
  ]

  # Wait for deployment to be ready
  wait = true
  timeout = 600

  depends_on = [digitalocean_kubernetes_cluster.planning_poker]
}

# Brief pause for CRD registration with API server
resource "time_sleep" "cert_manager_crds" {
  depends_on = [helm_release.cert_manager]
  create_duration = "60s"
}

# Cloudflare API token secret for DNS-01 challenges
resource "kubernetes_secret" "cloudflare_api_token" {
  metadata {
    name      = "cloudflare-api-token"
    namespace = "cert-manager"
  }

  type = "Opaque"

  data = {
    "api-token" = local.cloudflare_api_token
  }

  depends_on = [helm_release.cert_manager]
}

# ClusterIssuer for Let's Encrypt Production with DNS-01
resource "kubectl_manifest" "letsencrypt_prod" {
  yaml_body = yamlencode({
    apiVersion = "cert-manager.io/v1"
    kind       = "ClusterIssuer"
    metadata = {
      name = "letsencrypt-prod"
    }
    spec = {
      acme = {
        # Let's Encrypt ACME v2 production server
        server = "https://acme-v02.api.letsencrypt.org/directory"

        # Email for Let's Encrypt notifications
        email = var.letsencrypt_email

        # Secret to store the ACME account private key
        privateKeySecretRef = {
          name = "letsencrypt-prod"
        }

        # DNS-01 challenge solver using Cloudflare
        solvers = [
          {
            dns01 = {
              cloudflare = {
                apiTokenSecretRef = {
                  name = "cloudflare-api-token"
                  key  = "api-token"
                }
              }
            }
            selector = {
              dnsZones = ["hyperchad.dev"]
            }
          }
        ]
      }
    }
  })

  depends_on = [
    time_sleep.cert_manager_crds,
    kubernetes_secret.cloudflare_api_token
  ]
}

# ClusterIssuer for Let's Encrypt Staging (for non-prod environments)
resource "kubectl_manifest" "letsencrypt_staging" {
  count = terraform.workspace != "prod" ? 1 : 0

  yaml_body = yamlencode({
    apiVersion = "cert-manager.io/v1"
    kind       = "ClusterIssuer"
    metadata = {
      name = "letsencrypt-staging"
    }
    spec = {
      acme = {
        # Let's Encrypt ACME v2 staging server (for testing)
        server = "https://acme-v02.api.letsencrypt.org/directory"

        # Email for Let's Encrypt notifications
        email = var.letsencrypt_email

        # Secret to store the ACME account private key
        privateKeySecretRef = {
          name = "letsencrypt-staging"
        }

        # DNS-01 challenge solver using Cloudflare
        solvers = [
          {
            dns01 = {
              cloudflare = {
                apiTokenSecretRef = {
                  name = "cloudflare-api-token"
                  key  = "api-token"
                }
              }
            }
            selector = {
              dnsZones = ["hyperchad.dev"]
            }
          }
        ]
      }
    }
  })

  depends_on = [
    time_sleep.cert_manager_crds,
    kubernetes_secret.cloudflare_api_token
  ]
}
