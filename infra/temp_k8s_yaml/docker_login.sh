TOKEN=$(az acr login --name dystopiadev --expose-token --output tsv --query accessToken)
docker login dystopiadev.azurecr.io --username 00000000-0000-0000-0000-000000000000 --password-stdin <<< $TOKEN