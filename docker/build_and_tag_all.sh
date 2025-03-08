# dys-svc-webapp
docker build . -f dys-svc-webapp/Dockerfile -t dys-svc-webapp:latest
docker tag dys-svc-webapp:latest dystopiadev.azurecr.io/dys-svc-webapp:latest

# dys-svc-director
docker build . -f dys-svc-director/Dockerfile -t dys-svc-director:latest
docker tag dys-svc-director:latest dystopiadev.azurecr.io/dys-svc-director:latest

# dys-svc-auth
docker build . -f dys-svc-auth/Dockerfile -t dys-svc-auth:latest
docker tag dys-svc-auth:latest dystopiadev.azurecr.io/dys-svc-auth:latest