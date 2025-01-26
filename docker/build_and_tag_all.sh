# dys-svc-webapp
docker build . -f docker/dys-svc-webapp.Dockerfile -t dys-svc-webapp:latest
docker tag dys-svc-webapp:latest dystopiadev.azurecr.io/dys-svc-webapp:latest

# dys-svc-director
docker build . -f docker/dys-svc-director.Dockerfile -t dys-svc-director:latest
docker tag dys-svc-director:latest dystopiadev.azurecr.io/dys-svc-director:latest