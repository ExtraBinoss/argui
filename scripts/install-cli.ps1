# Install the latest verified Argui CLI release on Windows.
$ErrorActionPreference = 'Stop'
$repo = 'https://github.com/ExtraBinoss/argui'

if ($env:ARGUI_VERSION) {
    $version = $env:ARGUI_VERSION
} else {
    try {
        $version = (Invoke-RestMethod -Uri 'https://api.github.com/repos/ExtraBinoss/argui/releases/latest').tag_name
    } catch {
        throw 'Argui installer: could not find the latest release.'
    }
}
if ($version -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$') {
    throw "Argui installer: invalid release version: $version"
}

switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()) {
    'X64' { $target = 'x86_64-pc-windows-msvc' }
    default { throw 'Argui installer: this Windows architecture has no CLI release yet.' }
}

$asset = "argui-cli-$version-$target.zip"
$url = "$repo/releases/download/$version/$asset"
$temporary = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Path $temporary | Out-Null
try {
    $archive = Join-Path $temporary $asset
    $checksum = Join-Path $temporary "$asset.sha256"
    try {
        Invoke-WebRequest -Uri $url -OutFile $archive
        Invoke-WebRequest -Uri "$url.sha256" -OutFile $checksum
    } catch {
        throw "Argui installer: CLI release assets are unavailable for $version on $target. See $repo/releases"
    }
    $expected = ((Get-Content $checksum -Raw).Trim() -split '\s+')[0].ToLowerInvariant()
    if ($expected -notmatch '^[0-9a-f]{64}$') {
        throw 'Argui installer: invalid SHA-256 file.'
    }
    $actual = (Get-FileHash -Path $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) {
        throw 'Argui installer: checksum mismatch; installation stopped.'
    }
    Expand-Archive -Path $archive -DestinationPath $temporary
    $destination = if ($env:ARGUI_INSTALL_DIR) { $env:ARGUI_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Argui\bin' }
    New-Item -ItemType Directory -Force -Path $destination | Out-Null
    Copy-Item (Join-Path $temporary 'argui.exe') (Join-Path $destination 'argui.exe') -Force
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if (($userPath -split ';') -notcontains $destination) {
        [Environment]::SetEnvironmentVariable('Path', "$userPath;$destination", 'User')
    }
    $env:Path = "$destination;$env:Path"
    Write-Host "Installed Argui CLI $version at $destination\argui.exe"
    Write-Host 'Open a new terminal to use argui from any directory.'
} finally {
    Remove-Item -Recurse -Force $temporary -ErrorAction SilentlyContinue
}
