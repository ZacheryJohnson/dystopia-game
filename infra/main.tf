module "aks" {
  source = "./modules/aks"

  subscription_id = var.azure_subscription_id

  resource_groups = [
    {
      resource_group_name = "dystopia-dev",
      location = "West US"
      tags = {
        "deployment" = "terraform"
      }
    }
  ]

  k8s_clusters = [
    {
      cluster_name = "dystopia-dev"
      cluster_container_registry_name = "dystopiadev"
      resource_group_name = "dystopia-dev"
      dns_zone_base_domain = "determinism.dev"
      dns_zone_sub_domain = "dev"
      cert_manager_service_account_name = "cert-manager"
      cert_manager_namespace = "default"
      ingress_nginx_service_account_name = "ingress-nginx"
      ingress_nginx_namespace = "default"
      tags = {
        "deployment" = "terraform"
      }
    }
  ]

  container_registries = [
    {
      registry_name = "dystopiadev"
      sku = "Basic"
      resource_group_name = "dystopia-dev"
      tags = {
        "deployment" = "terraform"
      }
    }
  ]
}

module "otel" {
  source = "./modules/otel"

  kube_config = module.aks.kube_config[0]
  honeycomb_api_key = var.honeycomb_api_key
}

module "certs" {
  source = "./modules/certs"

  kube_config = module.aks.kube_config[0]

  acr_url                    = module.aks.acr_url[0]
  aks_resource_group_name    = module.aks.resource_group_name[0]
  aks_subscription_id        = var.azure_subscription_id
  aks_cert_manager_client_id = module.aks.cert_manager_client_id[0]
  public_ip                  = module.aks.lb_public_ip[0]
}

module "valkey" {
  source = "./modules/valkey"

  kube_config = module.aks.kube_config[0]
}

module "nats" {
  source = "./modules/nats"

  kube_config = module.aks.kube_config[0]
}