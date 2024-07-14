param (
    [string]$mode = ""
)

Set-Location frontend
npm install
if ($mode -eq "k8s") {
    npm run build-k8s
} else {
    npm run build
}
