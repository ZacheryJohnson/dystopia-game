param (
    [string]$working_dir = $(Get-Location)
)

Write-Output $(Get-Location)

Set-Location frontend
npm install
npm run build
Set-Location $working_dir
