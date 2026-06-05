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

# Bundle ffmpeg/ffprobe next to the app exe so thumbnails + clip metadata work
# out of the box. The Rust core's bundle-aware discovery finds binaries beside
# the executable, and the signtool loop below signs them with the rest.
# Prefer a vendored $env:SEDER_FFMPEG_DIR (reproducible); otherwise download a
# GPL static build (overridable via $env:SEDER_FFMPEG_WINDOWS_URL). Fail-soft
# unless $env:SEDER_REQUIRE_FFMPEG -eq "1".
$RequireFfmpeg = ($env:SEDER_REQUIRE_FFMPEG -eq "1")
try {
    $FfmpegSrc = $env:SEDER_FFMPEG_DIR
    if ($FfmpegSrc -and (Test-Path (Join-Path $FfmpegSrc "ffmpeg.exe"))) {
        Copy-Item (Join-Path $FfmpegSrc "ffmpeg.exe") (Join-Path $InstallDir "ffmpeg.exe") -Force
        Copy-Item (Join-Path $FfmpegSrc "ffprobe.exe") (Join-Path $InstallDir "ffprobe.exe") -Force
        Write-Host "Bundled ffmpeg/ffprobe from SEDER_FFMPEG_DIR"
    } else {
        $Url = if ($env:SEDER_FFMPEG_WINDOWS_URL) { $env:SEDER_FFMPEG_WINDOWS_URL } else { "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-full.zip" }
        $Zip = Join-Path $env:TEMP "seder-ffmpeg.zip"
        $Extract = Join-Path $env:TEMP "seder-ffmpeg-extract"
        if (Test-Path $Extract) { Remove-Item -Recurse -Force $Extract }
        Invoke-WebRequest -Uri $Url -OutFile $Zip -UseBasicParsing
        Expand-Archive -Path $Zip -DestinationPath $Extract -Force
        $ff = Get-ChildItem -Path $Extract -Recurse -Filter ffmpeg.exe | Select-Object -First 1
        $fp = Get-ChildItem -Path $Extract -Recurse -Filter ffprobe.exe | Select-Object -First 1
        if ($ff -and $fp) {
            Copy-Item $ff.FullName (Join-Path $InstallDir "ffmpeg.exe") -Force
            Copy-Item $fp.FullName (Join-Path $InstallDir "ffprobe.exe") -Force
            Write-Host "Bundled ffmpeg/ffprobe into $InstallDir"
        } else {
            throw "ffmpeg.exe/ffprobe.exe not found in downloaded archive"
        }
    }
} catch {
    if ($RequireFfmpeg) { throw }
    Write-Warning "Could not bundle ffmpeg ($_); continuing. The app still works if the host has ffmpeg on PATH."
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
