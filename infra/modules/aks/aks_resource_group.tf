resource "azurerm_resource_group" "rg" {
    for_each = {
        for rg in var.resource_groups:
            rg.resource_group_name => rg
    }

    name = each.key
    location = each.value.location
}

output "resource_group_name" {
    value = values(azurerm_resource_group.rg)[*].name
}