$ErrorActionPreference = "Stop"

$RootDir = Resolve-Path (Join-Path $PSScriptRoot "..")
$PackageName = (Select-String -Path (Join-Path $RootDir "Cargo.toml") -Pattern '^name = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value
$Version = $env:VERSION
if ([string]::IsNullOrWhiteSpace($Version)) {
    $Version = python (Join-Path $RootDir "scripts/package_common.py") version --root $RootDir
}
$Version = $Version.TrimStart("v")

$BuildDir = if ($env:BUILD_DIR) { $env:BUILD_DIR } else { Join-Path $RootDir "qt/build-release-windows" }
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { Join-Path $RootDir "dist/windows" }
$ArtifactDir = if ($env:ARTIFACT_DIR) { $env:ARTIFACT_DIR } else { Join-Path $RootDir "dist/artifacts" }

Remove-Item -Recurse -Force $BuildDir, $InstallDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $ArtifactDir, $InstallDir | Out-Null

python (Join-Path $RootDir "scripts/generate-icons.py") $RootDir


$ConfigureArgs = @(
    "-S", (Join-Path $RootDir "qt"),
    "-B", $BuildDir,
    "-DCMAKE_INSTALL_PREFIX=$InstallDir",
    "-DBUILD_TESTING=OFF"
)
if ($env:CMAKE_PREFIX_PATH) {
    $ConfigureArgs += "-DCMAKE_PREFIX_PATH=$env:CMAKE_PREFIX_PATH"
}
if ($env:CMAKE_GENERATOR) {
    $ConfigureArgs += @("-G", $env:CMAKE_GENERATOR)
}
if ($env:CMAKE_GENERATOR_PLATFORM) {
    $ConfigureArgs += @("-A", $env:CMAKE_GENERATOR_PLATFORM)
}
if (-not $env:CMAKE_GENERATOR -or $env:CMAKE_GENERATOR -eq "Ninja") {
    $ConfigureArgs += "-DCMAKE_BUILD_TYPE=Release"
}

cmake @ConfigureArgs
cmake --build $BuildDir --config Release
cmake --install $BuildDir --config Release

# Bundle ffmpeg/ffprobe for out-of-the-box thumbnail support.
$FfmpegDir = if ($env:SEDER_FFMPEG_DIR) { $env:SEDER_FFMPEG_DIR } else { $null }
if ($FfmpegDir -and (Test-Path $FfmpegDir)) {
    Write-Host "[fetch-ffmpeg] Copying from SEDER_FFMPEG_DIR=$FfmpegDir"
    foreach ($bin in @("ffmpeg.exe", "ffprobe.exe")) {
        $src = Join-Path $FfmpegDir $bin
        if (Test-Path $src) {
            Copy-Item $src (Join-Path $InstallDir $bin)
        }
    }
} else {
    Write-Host "[fetch-ffmpeg] Downloading Windows ffmpeg/ffprobe (GPL static)..."
    $ffUrl = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
    $tmp = New-TemporaryFile | ForEach-Object { $_.DirectoryName + "\" + [System.IO.Path]::GetRandomFileName() }
    New-Item -ItemType Directory $tmp | Out-Null
    try {
        $zipPath = "$tmp\ffmpeg.zip"
        Invoke-WebRequest -Uri $ffUrl -OutFile $zipPath -ErrorAction SilentlyContinue
        Expand-Archive -Path $zipPath -DestinationPath $tmp -ErrorAction SilentlyContinue
        foreach ($bin in @("ffmpeg.exe", "ffprobe.exe")) {
            $found = Get-ChildItem $tmp -Recurse -Filter $bin -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($found) { Copy-Item $found.FullName (Join-Path $InstallDir $bin) }
        }
    } catch {
        Write-Warning "[fetch-ffmpeg] Could not download ffmpeg: $_"
    } finally {
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    }
}

# Ad-hoc self-signed Authenticode signature for SEDER Productions identity.
# Does NOT clear SmartScreen — users still see "More info -> Run anyway" the
# first time. The cert is regenerated each run; for a stable cert across
# releases, store a .pfx in repo secrets and import it here instead.
$Cert = New-SelfSignedCertificate `
  -Subject "CN=SEDER Productions, O=SEDER Productions, C=GB" `
  -Type CodeSigningCert `
  -KeyUsage DigitalSignature `
  -KeyAlgorithm RSA -KeyLength 2048 `
  -CertStoreLocation Cert:\CurrentUser\My `
  -NotAfter (Get-Date).AddYears(5)

$SignTool = (Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match "x64\\signtool.exe$" } |
    Select-Object -First 1).FullName
if (-not $SignTool) { $SignTool = "signtool.exe" }

Get-ChildItem -Path $InstallDir -Recurse -Filter *.exe | ForEach-Object {
    & $SignTool sign /fd SHA256 /sha1 $Cert.Thumbprint `
        /tr http://timestamp.digicert.com /td SHA256 $_.FullName
    if ($LASTEXITCODE -ne 0) { throw "signtool failed for $($_.FullName)" }
}

$ArtifactName = python (Join-Path $RootDir "scripts/package_common.py") artifact-name --version $Version --platform windows-x64
$Artifact = Join-Path $ArtifactDir $ArtifactName
Remove-Item -Force $Artifact -ErrorAction SilentlyContinue
Compress-Archive -Path (Join-Path $InstallDir "*") -DestinationPath $Artifact

python (Join-Path $RootDir "scripts/package_common.py") checksums --artifact-dir $ArtifactDir | Out-Null
Write-Host "Packaged $Artifact"
