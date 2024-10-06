PROJECT_NAME="dystopia-dev"
export RESOURCE_GROUP_NAME="dystopia-dev"
MC_RESOURCE_GROUP_NAME="MC_dystopia-dev_dystopia-dev_westus"
REGISTRY_NAME="dystopiadev"
ACR_URL=$REGISTRY_NAME.azurecr.io
DNS_LABEL="dystopiadev"
CERT_MANAGER_REGISTRY=quay.io
CERT_MANAGER_TAG=v1.15.1
CERT_MANAGER_IMAGE_CONTROLLER=jetstack/cert-manager-controller
CERT_MANAGER_IMAGE_WEBHOOK=jetstack/cert-manager-webhook
CERT_MANAGER_IMAGE_CAINJECTOR=jetstack/cert-manager-cainjector
export NAMESPACE="default"

az aks get-credentials --resource-group $RESOURCE_GROUP_NAME --name $PROJECT_NAME

az acr import --name $REGISTRY_NAME --source $CERT_MANAGER_REGISTRY/$CERT_MANAGER_IMAGE_CONTROLLER:$CERT_MANAGER_TAG --image $CERT_MANAGER_IMAGE_CONTROLLER:$CERT_MANAGER_TAG
az acr import --name $REGISTRY_NAME --source $CERT_MANAGER_REGISTRY/$CERT_MANAGER_IMAGE_WEBHOOK:$CERT_MANAGER_TAG --image $CERT_MANAGER_IMAGE_WEBHOOK:$CERT_MANAGER_TAG
az acr import --name $REGISTRY_NAME --source $CERT_MANAGER_REGISTRY/$CERT_MANAGER_IMAGE_CAINJECTOR:$CERT_MANAGER_TAG --image $CERT_MANAGER_IMAGE_CAINJECTOR:$CERT_MANAGER_TAG

kubectl label namespace $NAMESPACE cert-manager.io/disable-validation=true

# Add the Jetstack Helm repository
helm repo add jetstack https://charts.jetstack.io

# Update your local Helm chart repository cache
helm repo update

# Install the cert-manager Helm chart
helm upgrade --install cert-manager jetstack/cert-manager \
  --namespace $NAMESPACE \
  --version=$CERT_MANAGER_TAG \
  --set crds.enabled=true \
  --set nodeSelector."kubernetes\.io/os"=linux \
  --set image.repository=$ACR_URL/$CERT_MANAGER_IMAGE_CONTROLLER \
  --set image.tag=$CERT_MANAGER_TAG \
  --set webhook.image.repository=$ACR_URL/$CERT_MANAGER_IMAGE_WEBHOOK \
  --set webhook.image.tag=$CERT_MANAGER_TAG \
  --set cainjector.image.repository=$ACR_URL/$CERT_MANAGER_IMAGE_CAINJECTOR \
  --set cainjector.image.tag=$CERT_MANAGER_TAG \
  --set-string podLabels."azure\.workload\.identity/use"=true \
  --set-string serviceAccount.labels."azure\.workload\.identity/use"=true

helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx

PUBLIC_IP=$(az network public-ip show --resource-group $MC_RESOURCE_GROUP_NAME -n $PROJECT_NAME-lb-public-ip --query ipAddress -o tsv)

helm upgrade --install ingress-nginx ingress-nginx/ingress-nginx \
  --namespace $NAMESPACE \
  --set controller.service.annotations."service\.beta\.kubernetes\.io/azure-dns-label-name"=$DNS_LABEL \
  --set controller.service.loadBalancerIP=$PUBLIC_IP \
  --set controller.service.externalTrafficPolicy=Local

echo "Sleeping 10 seconds to allow ingress-nginx resources to be created..."
sleep 10s

# Grafana
GRAFANA_NAMESPACE=grafana-alloy
helm repo add grafana https://grafana.github.io/helm-charts
helm repo update
kubectl create namespace $GRAFANA_NAMESPACE

read -p "Enter Grafana cloud user (all numeric): " GRAFANA_CLOUD_USER
read -p "Enter Grafana API token: " GRAFANA_CLOUD_API_TOKEN
export GRAFANA_CLOUD_USER
export GRAFANA_CLOUD_API_TOKEN

envsubst < config.alloy.template > config.alloy
kubectl create configmap --namespace $GRAFANA_NAMESPACE alloy-config "--from-file=config.alloy=./config.alloy"
rm config.alloy

helm install --namespace $GRAFANA_NAMESPACE alloy grafana/alloy -f alloy-values.yaml

kubectl apply -f webapp.yaml -n $NAMESPACE
kubectl apply -f director.yaml -n $NAMESPACE

export USER_ASSIGNED_CLIENT_ID="$(az identity show --resource-group $RESOURCE_GROUP_NAME --name $PROJECT_NAME-cert-manager --query 'clientId' -o tsv)"
export ZONE_NAME="dev.determinism.dev"
export SUBSCRIPTION_ID=$(az account show --query id)
export FQDN="dax.$ZONE_NAME"
export TLS_SECRET="dax-tls"

read -p "Enter email for cluster issuer: " EMAIL_ADDRESS
export EMAIL_ADDRESS

envsubst < cluster_issuer.yaml | kubectl apply -f -
envsubst < certificate.yaml | kubectl apply -n $NAMESPACE -f -
envsubst < ingress.yaml | kubectl apply -n $NAMESPACE -f -