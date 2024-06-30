resource "azurerm_kubernetes_cluster" "k8s" {
    for_each = {
        for cluster in var.k8s_clusters:
            cluster.cluster_name => cluster
    }

    name = each.key
    resource_group_name = each.value.resource_group_name
    location = azurerm_resource_group.rg[each.value.resource_group_name].location
    dns_prefix = "${each.key}"
    sku_tier = "Free"

    default_node_pool {
        name = "default"
        node_count = 1
        vm_size = "Standard_A2_v2"
    }

    identity {
        type = "SystemAssigned"
    }
}