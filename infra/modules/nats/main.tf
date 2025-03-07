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

resource "kubernetes_namespace" "nats" {
  metadata {
    name = "nats"
  }
}

resource "random_password" "nats_auth_token" {
  length = 32
  special = true
}

resource "kubernetes_secret_v1" "nats_auth_token" {
  metadata {
    name = "natstoken"
    namespace = "default"
  }

  data = {
    "token": random_password.nats_auth_token.result
  }
  type = "Opaque"
}

resource "kubernetes_secret_v1" "nats_auth_token_namespaced" {
  metadata {
    name = "natstoken"
    namespace = kubernetes_namespace.nats.id
  }

  data = {
    "token": random_password.nats_auth_token.result
  }
  type = "Opaque"
}

resource "helm_release" "prometheus_operator_crds" {
  chart = "prometheus-operator-crds"
  repository = "https://prometheus-community.github.io/helm-charts"
  name = "prometheusoperatorcrds"

  version = "18.0.1"
}

resource "helm_release" "nats" {
  chart = "nats"
  repository = "https://nats-io.github.io/k8s/helm/charts/"
  name  = "nats"

  version = "1.2.11"

  namespace = kubernetes_namespace.nats.id

  set {
    name  = "config.merge.authorization.token"
    value = random_password.nats_auth_token.result
  }

  set {
    name  = "promExporter.enabled"
    value = true
  }

  set {
    name  = "promExporter.podMonitor.enabled"
    value = true
  }

  depends_on = [
    helm_release.prometheus_operator_crds
  ]
}