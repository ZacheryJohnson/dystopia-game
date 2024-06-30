variable "resource_groups" {
    type = list(object({
        resource_group_name = string,
        location = string,
        tags = map(string)
    }))
}

variable "k8s_clusters" {
    type = list(object({
        cluster_name = string
        cluster_container_registry_name = string
        resource_group_name = string
        tags = map(string)
    }))
}

variable "container_registries" {
    type = list(object({
        registry_name = string
        sku = string
        resource_group_name = string
        tags = map(string)
    }))

    validation {
        condition = alltrue([for sku in var.container_registries[*].sku: contains(["Basic", "Standard", "Premium"], sku)])
        error_message = "Container registry SKU must be one of [\"Basic\", \"Standard\", \"Premium\"]"
    }
}