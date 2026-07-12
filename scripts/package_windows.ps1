param(
    [string]$Target = "x86_64-pc-windows-msvc",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $Root

$CargoToml = Get-Content "Cargo.toml"
$VersionLine = $CargoToml | Where-Object { $_ -match '^version = ' } | Select-Object -First 1
if (-not $VersionLine) {
    throw "Could not find version in Cargo.toml"
}
$Version = ($VersionLine -split '"')[1]
$Package = "bevy_open_siege-$Version-windows-x86_64"
$DistRoot = Join-Path $Root "dist"
$Dist = Join-Path $DistRoot $Package
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root "target" }
$ReleaseBinary = Join-Path $TargetDir "$Target\release\bevy_open_siege.exe"

if (-not $SkipBuild) {
    cargo build --release --target $Target
}
if (-not (Test-Path $ReleaseBinary)) {
    throw "release binary not found: $ReleaseBinary"
}

New-Item -ItemType Directory -Force $DistRoot | Out-Null

& $ReleaseBinary --validate-data
& $ReleaseBinary --audit-balance | Set-Content -Encoding utf8 (Join-Path $DistRoot "balance-audit.txt")
& $ReleaseBinary --audit-assets | Set-Content -Encoding utf8 (Join-Path $DistRoot "asset-audit.txt")
& $ReleaseBinary --audit-audio | Set-Content -Encoding utf8 (Join-Path $DistRoot "audio-audit.txt")
& $ReleaseBinary --audit-controls | Set-Content -Encoding utf8 (Join-Path $DistRoot "controls-audit.txt")
& $ReleaseBinary --audit-input-flow | Set-Content -Encoding utf8 (Join-Path $DistRoot "input-flow-audit.txt")
& $ReleaseBinary --audit-localization | Set-Content -Encoding utf8 (Join-Path $DistRoot "localization-audit.txt")
& $ReleaseBinary --audit-layout | Set-Content -Encoding utf8 (Join-Path $DistRoot "layout-audit.txt")
& $ReleaseBinary --audit-visual | Set-Content -Encoding utf8 (Join-Path $DistRoot "visual-readability-audit.txt")
& $ReleaseBinary --audit-accessibility | Set-Content -Encoding utf8 (Join-Path $DistRoot "accessibility-audit.txt")
& $ReleaseBinary --audit-performance | Set-Content -Encoding utf8 (Join-Path $DistRoot "performance-audit.txt")
& $ReleaseBinary --audit-privacy | Set-Content -Encoding utf8 (Join-Path $DistRoot "privacy-audit.txt")
& $ReleaseBinary --audit-release-provenance | Set-Content -Encoding utf8 (Join-Path $DistRoot "release-provenance-audit.txt")
& $ReleaseBinary --audit-marketing | Set-Content -Encoding utf8 (Join-Path $DistRoot "marketing-audit.txt")
& $ReleaseBinary --audit-ip | Set-Content -Encoding utf8 (Join-Path $DistRoot "ip-audit.txt")
& $ReleaseBinary --audit-save | Set-Content -Encoding utf8 (Join-Path $DistRoot "save-audit.txt")
& $ReleaseBinary --audit-playthrough | Set-Content -Encoding utf8 (Join-Path $DistRoot "playthrough-audit.txt")
& $ReleaseBinary --simulate-campaign | Set-Content -Encoding utf8 (Join-Path $DistRoot "campaign-simulation.txt")
& $ReleaseBinary --release-readiness | Set-Content -Encoding utf8 (Join-Path $DistRoot "release-readiness.txt")
& $ReleaseBinary --print-release-info | Set-Content -Encoding utf8 (Join-Path $DistRoot "release-info.txt")
python "$Root\scripts\generate_third_party_licenses.py" | Set-Content -Encoding utf8 (Join-Path $DistRoot "THIRD_PARTY_LICENSES.md")

Remove-Item -Recurse -Force $Dist -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force (Join-Path $Dist "assets") | Out-Null
Copy-Item $ReleaseBinary (Join-Path $Dist "bevy_open_siege.exe")

$Files = @(
    "README.md",
    "LICENSE",
    "CREDITS.md",
    "ART_ASSETS.md",
    "THIRD_PARTY_NOTICES.md",
    "STORE_PAGE.md",
    "STORE_SCREENSHOTS.md",
    "CONTENT_RATING.md",
    "PRESSKIT.md",
    "RELEASE_CHECKLIST.md",
    "RELEASE_NOTES.md",
    "QA_SIGNOFF.md",
    "PRIVACY.md",
    "SUPPORT.md",
    "TROUBLESHOOTING.md",
    "BUILD_PROVENANCE.md",
    "VERSION.ron"
)
foreach ($File in $Files) {
    Copy-Item (Join-Path $Root $File) $Dist
}

$Reports = @(
    "THIRD_PARTY_LICENSES.md",
    "release-info.txt",
    "balance-audit.txt",
    "asset-audit.txt",
    "audio-audit.txt",
    "controls-audit.txt",
    "input-flow-audit.txt",
    "localization-audit.txt",
    "layout-audit.txt",
    "visual-readability-audit.txt",
    "accessibility-audit.txt",
    "performance-audit.txt",
    "privacy-audit.txt",
    "release-provenance-audit.txt",
    "marketing-audit.txt",
    "ip-audit.txt",
    "save-audit.txt",
    "playthrough-audit.txt",
    "campaign-simulation.txt",
    "release-readiness.txt"
)
foreach ($Report in $Reports) {
    Copy-Item (Join-Path $DistRoot $Report) $Dist
}

Copy-Item -Recurse (Join-Path $Root "assets\manifest.ron") (Join-Path $Dist "assets")
Copy-Item -Recurse (Join-Path $Root "assets\art") (Join-Path $Dist "assets")
Copy-Item -Recurse (Join-Path $Root "assets\audio") (Join-Path $Dist "assets")
Copy-Item -Recurse (Join-Path $Root "assets\branding") (Join-Path $Dist "assets")
Copy-Item -Recurse (Join-Path $Root "assets\data") (Join-Path $Dist "assets")
Copy-Item -Recurse (Join-Path $Root "assets\i18n") (Join-Path $Dist "assets")
Copy-Item -Recurse (Join-Path $Root "assets\models") (Join-Path $Dist "assets")

Copy-Item (Join-Path $Root "scripts\manual_qa_session.sh") $Dist
Copy-Item (Join-Path $Root "scripts\platform_package_plan.sh") $Dist
Copy-Item (Join-Path $Root "scripts\qa_evidence_summary.sh") $Dist
Copy-Item (Join-Path $Root "scripts\final_signoff_check.sh") $Dist
Copy-Item (Join-Path $Root "scripts\verify_release.sh") $Dist
Copy-Item (Join-Path $Root "scripts\support_diagnostics.sh") $Dist
Copy-Item (Join-Path $Root "scripts\signoff_bundle.sh") $Dist
Copy-Item (Join-Path $Root "scripts\create_candidate_evidence.sh") $Dist
Copy-Item (Join-Path $Root "scripts\create_store_submission_pack.sh") $Dist
Copy-Item (Join-Path $Root "scripts\visual_smoke.sh") $Dist
Copy-Item (Join-Path $Root "scripts\store_screenshot_check.sh") $Dist
Copy-Item (Join-Path $Root "scripts\store_asset_audit.sh") $Dist
Copy-Item (Join-Path $Root "scripts\content_rating_audit.sh") $Dist
Copy-Item (Join-Path $Root "scripts\package_windows.ps1") $Dist
Copy-Item (Join-Path $Root "scripts\package_macos.sh") $Dist

& bash (Join-Path $Root "scripts\store_asset_audit.sh") $Dist | Set-Content -Encoding utf8 (Join-Path $Dist "store-asset-audit.txt")
& bash (Join-Path $Root "scripts\content_rating_audit.sh") $Dist | Set-Content -Encoding utf8 (Join-Path $Dist "content-rating-audit.txt")
& bash (Join-Path $Root "scripts\manual_qa_session.sh") --plan $Dist | Set-Content -Encoding utf8 (Join-Path $Dist "manual-qa-plan.txt")
& bash (Join-Path $Root "scripts\platform_package_plan.sh") --plan $Dist | Set-Content -Encoding utf8 (Join-Path $Dist "platform-package-plan.txt")
& bash (Join-Path $Root "scripts\final_signoff_check.sh") --plan $Dist | Set-Content -Encoding utf8 (Join-Path $Dist "final-signoff-plan.txt")

@"
runtime startup smoke pending: run bevy_open_siege.exe on the Windows release QA machine
window: pending
panic_scan: pending
"@ | Set-Content -Encoding utf8 (Join-Path $Dist "runtime-smoke.txt")

@"
visual startup smoke pending: capture a nonblank game screenshot on the Windows release QA machine
window: pending
screenshot: pending
panic_scan: pending
"@ | Set-Content -Encoding utf8 (Join-Path $Dist "visual-smoke.txt")

@"
audio startup smoke pending: run bevy_open_siege.exe --audio on the Windows release QA machine
window: pending
panic_scan: pending
"@ | Set-Content -Encoding utf8 (Join-Path $Dist "audio-smoke.txt")

python "$Root\scripts\generate_release_manifest.py" $Dist "windows-x86_64" | Set-Content -Encoding utf8 (Join-Path $Dist "release-manifest.json")

$Hashes = Get-ChildItem $Dist -Recurse -File |
    Where-Object { $_.Name -ne "SHA256SUMS" } |
    Sort-Object FullName |
    ForEach-Object {
        $Relative = $_.FullName.Substring($Dist.Length + 1).Replace("\", "/")
        $Hash = (Get-FileHash -Algorithm SHA256 $_.FullName).Hash.ToLowerInvariant()
        "$Hash  $Relative"
    }
$Hashes | Set-Content -Encoding ascii (Join-Path $Dist "SHA256SUMS")

$Archive = Join-Path $DistRoot "$Package.zip"
Remove-Item -Force $Archive -ErrorAction SilentlyContinue
Compress-Archive -Path $Dist -DestinationPath $Archive

Write-Output "Created dist/$Package.zip"
