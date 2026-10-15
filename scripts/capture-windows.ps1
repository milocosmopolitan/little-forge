param([string]$Executable, [string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force $OutputDirectory | Out-Null
$env:WGPU_BACKEND = 'dx12'

function Capture-Scene([string]$Scene, [bool]$Control) {
  $name = "static-dxc-$Scene"
  if ($Control) {
    $env:LITTLE_FORGE_QA_UNPATCHED_SHADER = '1'
    $name = "unpatched-$name"
  } else {
    Remove-Item Env:LITTLE_FORGE_QA_UNPATCHED_SHADER -ErrorAction SilentlyContinue
  }
  $png = "$OutputDirectory/$name.png"
  $p = Start-Process $Executable -ArgumentList @('--safe-window','--qa',$Scene,'--screenshot',$png) -PassThru -RedirectStandardOutput "$OutputDirectory/$name.stdout.log" -RedirectStandardError "$OutputDirectory/$name.stderr.log"
  if (!$p.WaitForExit(90000)) { $p.Kill(); throw "Capture timeout: $name" }
  if (!(Test-Path $png)) { throw "Missing capture: $name" }
  if (Select-String -Path "$OutputDirectory/$name.stderr.log" -Pattern 'panicked|\bERROR\b') { throw "Renderer error: $name" }
  python scripts/compare-images.py "docs/qa/golden/linux-llvmpipe/$Scene.png" $png --diff "$OutputDirectory/$name-diff.png" > "$OutputDirectory/$name-comparison.json"
  $comparisonExit = $LASTEXITCODE
  Get-Content "$OutputDirectory/$name-comparison.json"
  if ($Control) {
    if ($comparisonExit -eq 0) { throw 'Unpatched control no longer reproduces the defect; review whether workaround can be removed' }
  } elseif ($comparisonExit -ne 0) { throw "Windows scene differs from reviewed reference: $name" }
}

# A/B control uses the same executable, device, camera, assets and scene coordinates.
Capture-Scene -Scene ambient -Control $true
foreach ($scene in @('ambient','sale','expanded')) {
  Capture-Scene -Scene $scene -Control $false
}
