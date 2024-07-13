resource "azurerm_container_registry" "container_registry" {
    for_each = {
        for container_registry in var.container_registries:
            container_registry.registry_name => container_registry
    }

    name = each.key
    sku = each.value.sku
    resource_group_name = each.value.resource_group_name
    location = azurerm_resource_group.rg[each.value.resource_group_name].location
}

resource "azurerm_role_assignment" "attach_to_k8s" {
    for_each = {
        for container_registry in var.container_registries:
            container_registry.registry_name => container_registry
    }

    principal_id                     = azurerm_kubernetes_cluster.k8s["dystopia-dev"].kubelet_identity[0].object_id
    role_definition_name             = "AcrPull"
    scope                            = azurerm_container_registry.container_registry[each.key].id
    skip_service_principal_aad_check = true
}