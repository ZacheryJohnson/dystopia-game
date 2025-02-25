resource "azurerm_kubernetes_cluster" "k8s" {
    for_each = {
        for cluster in var.k8s_clusters:
            cluster.cluster_name => cluster
    }

    name = each.key
    resource_group_name = each.value.resource_group_name
    location = azurerm_resource_group.rg[each.value.resource_group_name].location
    dns_prefix = each.key
    sku_tier = "Free"

    oidc_issuer_enabled = true
    workload_identity_enabled = true

    default_node_pool {
        name = "default"
        node_count = 1
        vm_size = "Standard_F2s_v2"
        os_sku = "Ubuntu"
    }

    key_vault_secrets_provider {
        secret_rotation_enabled = true
    }

    identity {
        type = "SystemAssigned"
    }
}

output "kube_config" {
    value = values(azurerm_kubernetes_cluster.k8s)[*].kube_config
}
