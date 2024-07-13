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