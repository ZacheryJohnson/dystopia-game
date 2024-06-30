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