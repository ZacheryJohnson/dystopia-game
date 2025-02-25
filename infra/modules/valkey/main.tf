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

resource "kubernetes_namespace" "valkey" {
  metadata {
    name = "valkey"
  }
}

resource "random_password" "valkey_default_password" {
  length = 24
  special = true
}

resource "kubernetes_secret_v1" "valkey_default_password_default" {
  metadata {
    name = "valkeypass"
    namespace = "default"
  }

  data = {
    "default_password": random_password.valkey_default_password.result
  }
  type = "Opaque"
}

resource "kubernetes_secret_v1" "valkey_default_password_valkey" {
  metadata {
    name = "valkeypass"
    namespace = kubernetes_namespace.valkey.id
  }

  data = {
    "default_password": random_password.valkey_default_password.result
  }
  type = "Opaque"
}

resource "helm_release" "valkey" {
  chart = "valkey"
  repository = "oci://registry-1.docker.io/bitnamicharts/"
  name  = "valkey"

  version = "2.4.0"

  namespace = kubernetes_namespace.valkey.id

  set {
    name  = "auth.password"
    value = random_password.valkey_default_password.result
  }
}