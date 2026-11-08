param(
  [Parameter(Mandatory=$true)][string]$HostExecutable,
  [Parameter(Mandatory=$true)][string]$ExtensionId
)
$ErrorActionPreference = 'Stop'
if (-not (Test-Path -LiteralPath $HostExecutable -PathType Leaf)) { throw "Native host executable not found: $HostExecutable" }
if ($ExtensionId -notmatch '^[a-p]{32}$') { throw 'Extension ID must be the 32-character ID shown on chrome://extensions.' }
$hostPath = (Resolve-Path -LiteralPath $HostExecutable).Path
$manifestDir = Join-Path $env:LOCALAPPDATA 'UniversalBrowserSync\NativeMessaging'
New-Item -ItemType Directory -Force -Path $manifestDir | Out-Null
$manifestPath = Join-Path $manifestDir 'com.universal-browser-sync.bridge.json'
$manifest = @{
  name = 'com.universal-browser-sync.bridge'
  description = 'Universal Browser Sync metadata-only native messaging bridge'
  path = $hostPath
  type = 'stdio'
  allowed_origins = @("chrome-extension://$ExtensionId/")
} | ConvertTo-Json -Depth 4
[System.IO.File]::WriteAllText($manifestPath, $manifest, [System.Text.UTF8Encoding]::new($false))
$keys = @(
  'HKCU:\Software\Google\Chrome\NativeMessagingHosts\com.universal-browser-sync.bridge',
  'HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\com.universal-browser-sync.bridge',
  'HKCU:\Software\BraveSoftware\Brave-Browser\NativeMessagingHosts\com.universal-browser-sync.bridge',
  'HKCU:\Software\Opera Software\NativeMessagingHosts\com.universal-browser-sync.bridge'
)
foreach ($key in $keys) {
  New-Item -Path $key -Force | Out-Null
  Set-Item -Path $key -Value $manifestPath
}
Write-Host "Registered UBS native host for Chrome, Edge, Brave, and Opera."
Write-Host "Manifest: $manifestPath"
Write-Host 'Restart the browser after loading the UBS Helper extension.'
