param([string]$Executable, [string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force $OutputDirectory | Out-Null
$dxc = Get-ChildItem 'C:/Program Files (x86)/Windows Kits/10/bin' -Recurse -Filter dxcompiler.dll | Where-Object { $_.Directory.Name -eq 'x64' } | Sort-Object FullName -Descending | Select-Object -First 1
if (!$dxc) { throw 'Missing SDK DXC' }
Copy-Item $dxc.FullName . -Force
Copy-Item (Join-Path $dxc.Directory.FullName 'dxil.dll') . -Force
$env:WGPU_BACKEND = 'dx12'
$env:LITTLE_FORGE_SPRITE_DIAGNOSTICS = '1'
$matchingCaptureCount = 0
foreach ($compiler in @('dxc', 'fxc')) {
  $env:WGPU_DX12_COMPILER = $compiler
  foreach ($mode in @('original','dot','explicit')) {
    $env:LITTLE_FORGE_SPRITE_SHADER = $mode
    $name = "$compiler-$mode"
    $png = "$OutputDirectory/$name.png"
    $p = Start-Process $Executable -ArgumentList @('--safe-window','--qa','ambient','--screenshot',$png) -PassThru -RedirectStandardOutput "$OutputDirectory/$name.stdout.log" -RedirectStandardError "$OutputDirectory/$name.stderr.log"
    if (!$p.WaitForExit(90000)) { $p.Kill(); "timeout: $name" | Out-File "$OutputDirectory/results.txt" -Append; continue }
    if (!(Test-Path $png)) { "missing capture: $name" | Out-File "$OutputDirectory/results.txt" -Append; continue }
    if (Select-String -Path "$OutputDirectory/$name.stderr.log" -Pattern 'panicked|\bERROR\b') { "renderer failed: $name" | Out-File "$OutputDirectory/results.txt" -Append; continue }
    python scripts/compare-images.py docs/qa/golden/linux-llvmpipe/ambient.png $png --diff "$OutputDirectory/$name-diff.png" > "$OutputDirectory/$name-comparison.json"
    if ($LASTEXITCODE -eq 0) { $matchingCaptureCount += 1 }
    "${name}: comparison exit=$LASTEXITCODE" | Out-File "$OutputDirectory/results.txt" -Append
  }
}
if ($matchingCaptureCount -eq 0) { throw 'No shader variant matched the reviewed scene; investigation remains open' }
