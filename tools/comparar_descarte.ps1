param([ValidateRange(11,1000)][int]$Frames = 30)
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath (Split-Path -Parent $PSScriptRoot)
$folder = Join-Path $PWD 'target/culling-bench'
New-Item -ItemType Directory -Force -Path $folder | Out-Null
$results = foreach ($scene in @(
    @{Name='Tierra'; Args=@('--planet','3')},
    @{Name='Venus'; Args=@('--venus')},
    @{Name='Luna'; Args=@('--walking')}
)) {
    $rows = @()
    foreach ($mode in @('on','off')) {
        $extra = @()
        if ($mode -eq 'off') { $extra = @('--no-object-culling') }
        $capture = Join-Path $folder ($scene.Name + '-' + $mode + '.png')
        $lines = & './target/release/rana-solar.exe' @($scene.Args) @extra --hidden --no-audio --freeze-scene --frames $Frames --capture $capture
        if ($LASTEXITCODE -ne 0) { throw ('Error al ejecutar ' + $scene.Name) }
        $lines | Set-Content -LiteralPath (Join-Path $folder ($scene.Name + '-' + $mode + '.log'))
        $metric = $lines | Select-String '^Culling: enabled=(true|false) objects=(\d+) rejected=(\d+) avoided_triangles=(\d+) submitted_triangles=(\d+) raster_triangles=(\d+)'
        if (-not $metric) { throw 'Faltan contadores de descarte' }
        $groups = $metric.Matches[0].Groups
        $rows += [pscustomobject]@{Escena=$scene.Name; Descarte=$mode; Objetos=[int]$groups[2].Value; Rechazados=[int]$groups[3].Value; TriangulosEvitados=[int]$groups[4].Value; TriangulosRecorte=[int]$groups[5].Value; TriangulosRaster=[int]$groups[6].Value; ImagenSHA256=(Get-FileHash -LiteralPath $capture -Algorithm SHA256).Hash}
    }
    if ($rows[0].ImagenSHA256 -ne $rows[1].ImagenSHA256) { throw ('Las imagenes difieren en ' + $scene.Name) }
    if ($rows[0].TriangulosRecorte -ge $rows[1].TriangulosRecorte) { throw ('No se redujo el trabajo en ' + $scene.Name) }
    $rows
}
$results | Export-Csv -NoTypeInformation -Encoding utf8 -LiteralPath (Join-Path $folder 'comparacion.csv')
$results | Select-Object Escena,Descarte,Objetos,Rechazados,TriangulosEvitados,TriangulosRecorte,TriangulosRaster | Format-Table -AutoSize
Write-Output 'Verificado: imagenes PNG identicas por SHA256 y menos triangulos procesados con descarte en las tres escenas.'
Write-Output 'El interruptor afecta al descarte por objeto. El recorte por triangulo y descarte de caras de esferas permanecen activos.'
