data "azurerm_dns_zone" "base_domain" {
    for_each = {
        for cluster in var.k8s_clusters:
            cluster.cluster_name => cluster
    }

    name                = each.value.dns_zone_base_domain
    resource_group_name = "ManuallyManagedResources"
}

# We need our DNS zone to be in the same resource group as everything else
# There's no explicit support for child zones in terraform, so just append to the existing name
resource "azurerm_dns_zone" "domain" {
    for_each = {
        for cluster in var.k8s_clusters:
            cluster.cluster_name => cluster
    }
    name                = "${each.value.dns_zone_sub_domain}.${data.azurerm_dns_zone.base_domain[each.key].name}"
    resource_group_name = each.value.resource_group_name
}

# create ns record for sub-zone in parent zone
resource "azurerm_dns_ns_record" "child_zone" {
    for_each = {
        for cluster in var.k8s_clusters:
            cluster.cluster_name => cluster
    }
    name                = each.value.dns_zone_sub_domain
    zone_name           = data.azurerm_dns_zone.base_domain[each.key].name
    resource_group_name = data.azurerm_dns_zone.base_domain[each.key].resource_group_name
    ttl                 = 60

    records = azurerm_dns_zone.domain[each.key].name_servers
}

resource "azurerm_public_ip" "lb_ingress_ip" {
    for_each = {
        for cluster in var.k8s_clusters:
            cluster.cluster_name => cluster
    }

    name = "${each.key}-lb-public-ip"
    resource_group_name = azurerm_kubernetes_cluster.k8s[each.key].node_resource_group
    location = azurerm_resource_group.rg[each.value.resource_group_name].location
    allocation_method = "Static"
    sku = "Standard"
    
    lifecycle {
        create_before_destroy = true
    }
}

resource "azurerm_dns_a_record" "webapp_dns" {
    for_each = {
        for cluster in var.k8s_clusters:
            cluster.cluster_name => cluster
    }

    name                = "dax" # TODO: this should be read from each cluster
    zone_name           = azurerm_dns_zone.domain[each.key].name
    resource_group_name = each.value.resource_group_name
    ttl                 = 300
    records             = [azurerm_public_ip.lb_ingress_ip[each.key].ip_address]
}