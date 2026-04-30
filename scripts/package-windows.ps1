$ErrorActionPreference = "Stop"

$RootDir = Resolve-Path (Join-Path $PSScriptRoot "..")
$Version = $env:VERSION
if ([string]::IsNullOrWhiteSpace($Version)) {
    $Version = (Select-String -Path (Join-Path $RootDir "Cargo.toml") -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value
}
$Version = $Version.TrimStart("v")

$BuildDir = if ($env:BUILD_DIR) { $env:BUILD_DIR } else { Join-Path $RootDir "qt/build-release-windows" }
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { Join-Path $RootDir "dist/windows" }
$ArtifactDir = if ($env:ARTIFACT_DIR) { $env:ARTIFACT_DIR } else { Join-Path $RootDir "dist/artifacts" }

Remove-Item -Recurse -Force $BuildDir, $InstallDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $ArtifactDir, $InstallDir | Out-Null

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

$Artifact = Join-Path $ArtifactDir "seder-dit-tool-v$Version-windows-x64.zip"
Remove-Item -Force $Artifact -ErrorAction SilentlyContinue
Compress-Archive -Path (Join-Path $InstallDir "*") -DestinationPath $Artifact

Write-Host "Packaged $Artifact"
