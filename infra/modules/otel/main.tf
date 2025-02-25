variable "kube_config" {
  type = any
}

provider "helm" {
  kubernetes {
    host                   = var.kube_config[0].host
    username               = var.kube_config[0].username
    password               = var.kube_config[0].password
    client_certificate     = base64decode(var.kube_config[0].client_certificate)
    client_key             = base64decode(var.kube_config[0].client_key)
    cluster_ca_certificate = base64decode(var.kube_config[0].cluster_ca_certificate)
  }
}

provider "kubernetes" {
  host                   = var.kube_config[0].host
  username               = var.kube_config[0].username
  password               = var.kube_config[0].password
  client_certificate     = base64decode(var.kube_config[0].client_certificate)
  client_key             = base64decode(var.kube_config[0].client_key)
  cluster_ca_certificate = base64decode(var.kube_config[0].cluster_ca_certificate)
}

variable "honeycomb_api_key" {
  type = string
  description = "Can be found at https://ui.honeycomb.io/<project_name>/environments/test/send-data#"
}

resource "kubernetes_namespace" "otel" {
  metadata {
    name = "otel"
  }
}

resource "kubernetes_secret_v1" "honeycomb_api_key" {
  metadata {
    name = "honeycomb"
    namespace = kubernetes_namespace.otel.id
  }

  data = {
    "api-key": var.honeycomb_api_key
  }
  type = "Opaque"
}

resource "helm_release" "otel_collector" {
  name       = "otel-controller"
  repository = "https://open-telemetry.github.io/opentelemetry-helm-charts"
  chart      = "opentelemetry-collector"

  namespace = kubernetes_namespace.otel.id

  values = [ file("${path.module}/values-daemonset.yaml") ]

  set {
    name  = "config.exporters.otlp.headers.x-honeycomb-team"
    value = var.honeycomb_api_key
  }

  set {
    name  = "config.exporters.otlp/k8s-metrics.headers.x-honeycomb-team"
    value = var.honeycomb_api_key
  }

  set {
    name  = "config.exporters.otlp/k8s-logs.headers.x-honeycomb-team"
    value = var.honeycomb_api_key
  }
}