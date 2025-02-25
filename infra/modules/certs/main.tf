locals {
  acr_name = split(".", var.acr_url)[0]
  controller_image_path = "${var.cert_manager_image_controller}:${var.cert_manager_image_version}"
  cainjector_image_path = "${var.cert_manager_image_cainjector}:${var.cert_manager_image_version}"
  webhook_image_path = "${var.cert_manager_image_webhook}:${var.cert_manager_image_version}"
}

variable "acr_url" {
  type = string
}

variable "public_ip" {
  type = string
}

variable "cert_manager_registry" {
  type = string
  default = "quay.io/jetstack"
}

variable "cert_manager_image_version" {
  type    = string
  default = "v1.17.0"
}

variable "cert_manager_image_controller" {
  type    = string
  default = "cert-manager-controller"
}

variable "cert_manager_image_cainjector" {
  type    = string
  default = "cert-manager-cainjector"
}

variable "cert_manager_image_webhook" {
  type    = string
  default = "cert-manager-webhook"
}

variable "aks_cert_manager_client_id" {
  type = string
  description = "az identity show --resource-group $RESOURCE_GROUP_NAME --name $NAME --query 'clientId' -o tsv"
}

variable "aks_subscription_id" {
  type = string
  description = "az account show --query id"
}

variable "aks_resource_group_name" {
  type = string
}

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

resource "helm_release" "cert_manager" {
  name       = "cert-manager"
  repository = "https://charts.jetstack.io"
  chart      = "cert-manager"

  version = var.cert_manager_image_version

  set {
    name  = "crds.enabled"
    value = "true"
  }

  set {
    name  = "nodeSelector.kubernetes\\.io/os"
    value = "linux"
  }

  set {
    name  = "image.repository"
    value = "${var.acr_url}/${var.cert_manager_image_controller}"
  }

  set {
    name  = "image.tag"
    value = var.cert_manager_image_version
  }

  set {
    name  = "webhook.image.repository"
    value = "${var.acr_url}/${var.cert_manager_image_webhook}"
  }

  set {
    name  = "webhook.image.tag"
    value = var.cert_manager_image_version
  }

  set {
    name  = "cainjector.image.repository"
    value = "${var.acr_url}/${var.cert_manager_image_cainjector}"
  }

  set {
    name  = "cainjector.image.tag"
    value = var.cert_manager_image_version
  }

  set {
    name  = "podLabels.azure\\.workload\\.identity/use"
    value = "true"
    type  = "string"
  }

  set {
    name  = "serviceAccount.labels.azure\\.workload\\.identity/use"
    value = "true"
    type  = "string"
  }
}

resource "helm_release" "nginx" {
  name       = "nginx"
  repository = "https://kubernetes.github.io/ingress-nginx"
  chart      = "ingress-nginx"

  set {
    name  = "controller.service.annotations.service\\.beta\\.kubernetes\\.io/azure-dns-label-name"
    value = "dystopiadev"
  }

  set {
    name  = "controller.service.loadBalancerIP"
    value = var.public_ip
  }

  set {
    name  = "controller.service.externalTrafficPolicy"
    value = "Local"
  }
}

resource "helm_release" "certs" {
  name  = "certs"
  chart = "${path.module}/chart"

  set {
    name  = "aks.resourceGroupName"
    value = var.aks_resource_group_name
  }

  set {
    name  = "aks.subscriptionId"
    value = var.aks_subscription_id
  }

  set {
    name  = "aks.userClientId"
    value = var.aks_cert_manager_client_id
  }

  depends_on = [
    // We need the CRDs in the cert manager chart before applying our resources
    helm_release.cert_manager
  ]
}

resource "null_resource" "push_cert_manager_images" {
  provisioner "local-exec" {
    command = <<-EOF
      az acr import --name ${local.acr_name} --source ${var.cert_manager_registry}/${local.controller_image_path} --image ${local.controller_image_path}
      az acr import --name ${local.acr_name} --source ${var.cert_manager_registry}/${local.cainjector_image_path} --image ${local.cainjector_image_path}
      az acr import --name ${local.acr_name} --source ${var.cert_manager_registry}/${local.webhook_image_path} --image ${local.webhook_image_path}
    EOF

    interpreter = ["bash", "-c"]
  }
}