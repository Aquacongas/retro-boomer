# Optional Authenticode signing using a real, pre-provisioned CurrentUser/My certificate.
param(
    [Parameter(Mandatory = $true, ParameterSetName = "Binaries")]
    [string] $BinDir,
    [Parameter(Mandatory = $true, ParameterSetName = "File")]
    [string] $File
)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$Thumbprint = $env:RETRO_BOOMER_WINDOWS_SIGN_THUMBPRINT
$Timestamp = $env:RETRO_BOOMER_WINDOWS_TIMESTAMP_URL
if ($Thumbprint -notmatch '^[0-9a-fA-F]{40}$') { throw "A real signing certificate thumbprint is required" }
if (-not $Timestamp -or -not $Timestamp.StartsWith("https://")) { throw "An owner-approved HTTPS timestamp service is required" }
$Certificate = Get-Item "Cert:\CurrentUser\My\$Thumbprint"
if (-not $Certificate.HasPrivateKey) { throw "The signing certificate has no private key" }
$Tool = Get-Command signtool.exe -ErrorAction SilentlyContinue
if ($Tool) {
    $SignTool = $Tool.Source
} else {
    $Sdk = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"
    $Candidates = @(Get-ChildItem "$Sdk\*\x64\signtool.exe" | Sort-Object FullName -Descending)
    if ($Candidates.Count -eq 0) { throw "Windows SDK signtool.exe was not found" }
    $SignTool = $Candidates[0].FullName
}
$Paths = @($File)
if ($PSCmdlet.ParameterSetName -eq "Binaries") {
    $Paths = @("retro-boomerd.exe", "retro-boomer-cli.exe", "retro-boomer-miner.exe", "retro-boomer-gui.exe") | ForEach-Object { Join-Path $BinDir $_ }
}
foreach ($Path in $Paths) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "Signing input is missing: $Path" }
    & $SignTool sign /s My /sha1 $Thumbprint /fd SHA256 /tr $Timestamp /td SHA256 $Path
    if ($LASTEXITCODE -ne 0) { throw "Authenticode signing failed" }
    & $SignTool verify /pa $Path
    if ($LASTEXITCODE -ne 0) { throw "Authenticode verification failed" }
}
