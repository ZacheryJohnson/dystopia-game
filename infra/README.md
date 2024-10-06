# Infra Setup Steps

## Prerequisites
Assumes Windows.

Install Git Bash.
Install Chocolatey.
Install Docker Desktop.
Install kubectl: `choco install kubernetes-cli`
Install helm: `choco install kubernetes-helm`

## App Build Steps
From root directory: `./docker/build_and_tag_all.sh`

## Infrastructure Steps
Using Git Bash:
1. `cd infra`
2. `terraform init`
3. `terraform apply` -> enter "yes" when prompted
4. Ensure successful terraform apply: `Apply complete! Resources: 14 added, 0 changed, 0 destroyed.`
5. `cd temp_k8s_yaml`
6. `./docker_login.sh`
7. `../../docker/push_all.sh`
8. `./install.sh`