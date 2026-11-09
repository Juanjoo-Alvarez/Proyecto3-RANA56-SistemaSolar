param(
    [ValidateRange(11, 10000)][int]$Frames = 120,
    [ValidateRange(1, 20)][int]$Repeticiones = 3
)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$game = Join-Path $projectRoot 'target/release/rana-solar.exe'
if (-not (Test-Path -LiteralPath $game)) { throw 'Compila primero con cargo build --release --offline.' }
$outputFolder = Join-Path $projectRoot 'target/surface-bench'
New-Item -ItemType Directory -Force -Path $outputFolder | Out-Null
$results = foreach ($scene in @(
    @{ Name = 'Venus'; Flag = '--venus' },
    @{ Name = 'Mercurio'; Flag = '--mercury' },
    @{ Name = 'Luna'; Flag = '--walking' }
)) {
    $samples = @()
    for ($run = 1; $run -le $Repeticiones; $run++) {
        $capture = Join-Path $outputFolder ($scene.Name + '-final.png')
        $lines = & $game --hidden --no-audio $scene.Flag --frames $Frames --capture $capture
        if ($LASTEXITCODE -ne 0) { throw "Fallo al medir $($scene.Name)" }
        $lines | Set-Content -LiteralPath (Join-Path $outputFolder ($scene.Name + '-' + $run + '.log'))
        $metric = $lines | Select-String 'Cuadro completo: media ([\d.]+) ms'
        if (-not $metric) { throw 'No se encontro la medicion del cuadro completo.' }
        $samples += [double]::Parse($metric.Matches[0].Groups[1].Value, [Globalization.CultureInfo]::InvariantCulture)
    }
    $sorted = @($samples | Sort-Object)
    $middle = [int][math]::Floor($sorted.Count / 2)
    $median = if ($sorted.Count % 2) { $sorted[$middle] } else { ($sorted[$middle - 1] + $sorted[$middle]) / 2 }
    [pscustomobject]@{ Escenario = $scene.Name; CuadrosPorMuestra = $Frames; Muestras = $Repeticiones; MedianaMs = $median; FPS = [math]::Round(1000 / $median, 1); MinMs = $sorted[0]; MaxMs = $sorted[-1] }
}
$results | Format-Table -AutoSize
Write-Output 'Tiempo de cuadro completo (incluye presentacion, HUD y retratos; sin audio), sin primeros 10 cuadros. Ejecutar sin otras compilaciones o pruebas simultaneas.'
