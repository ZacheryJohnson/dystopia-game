terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "=4.20.0"
    }

    helm = {
      source  = "hashicorp/helm"
      version = "=2.17.0"
    }

    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "=2.35.1"
    }
  }
}