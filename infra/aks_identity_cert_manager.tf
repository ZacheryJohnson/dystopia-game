resource "azurerm_user_assigned_identity" "cert_manager" {
    for_each = {
        for cluster in var.k8s_clusters:
            cluster.cluster_name => cluster
    }

    name = "${each.key}-cert-manager"
    location = azurerm_resource_group.rg[each.value.resource_group_name].location
    resource_group_name = each.value.resource_group_name
}

resource "azurerm_role_assignment" "cert_manager_dns_zone" {
    for_each = {
        for cluster in var.k8s_clusters:
            cluster.cluster_name => cluster
    }

    principal_id         = azurerm_user_assigned_identity.cert_manager[each.key].principal_id
    scope                = azurerm_dns_zone.domain[each.key].id
    role_definition_name = "DNS Zone Contributor"
}

resource "azurerm_role_assignment" "cert_manager_key_vault" {
    for_each = {
        for cluster in var.k8s_clusters:
            cluster.cluster_name => cluster
    }

    principal_id         = azurerm_user_assigned_identity.cert_manager[each.key].principal_id
    scope                = azurerm_key_vault.dystopia[each.key].id
    role_definition_name = "Key Vault Certificate User"
}

resource "azurerm_federated_identity_credential" "cert_manager" {
    for_each = {
        for cluster in var.k8s_clusters:
            cluster.cluster_name => cluster
    }

    name                = "${each.key}-cert-manager"
    resource_group_name = each.value.resource_group_name
    audience            = ["api://AzureADTokenExchange"]
    issuer              = azurerm_kubernetes_cluster.k8s[each.key].oidc_issuer_url
    parent_id           = azurerm_user_assigned_identity.cert_manager[each.key].id
    subject             = "system:serviceaccount:${each.value.cert_manager_namespace}:${each.value.cert_manager_service_account_name}"
}
