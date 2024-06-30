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