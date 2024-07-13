data "azurerm_client_config" "current" {}

resource "azurerm_key_vault" "dystopia" {
    for_each = {
        for cluster in var.k8s_clusters:
            cluster.cluster_name => cluster
    }

    name                        = each.key
    resource_group_name         = each.value.resource_group_name
    location                    = azurerm_resource_group.rg[each.value.resource_group_name].location
    enabled_for_disk_encryption = true
    tenant_id                   = data.azurerm_client_config.current.tenant_id
    soft_delete_retention_days  = 7
    purge_protection_enabled    = false

    sku_name = "standard"

    enable_rbac_authorization = true
}

resource "azurerm_key_vault_access_policy" "client" {
    for_each = {
        for cluster in var.k8s_clusters:
            cluster.cluster_name => cluster
    }

    key_vault_id = azurerm_key_vault.dystopia[each.key].id
    tenant_id    = data.azurerm_client_config.current.tenant_id
    object_id    = data.azurerm_client_config.current.object_id

    key_permissions = [
        "Get",
    ]

    secret_permissions = [
        "Get",
    ]

    certificate_permissions = [
        "Get", "Import"
    ]

    storage_permissions = [
        "Get",
    ]
}