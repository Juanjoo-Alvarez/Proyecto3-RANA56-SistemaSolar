$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath (Split-Path -Parent $PSScriptRoot)
$env:RANA_FFMPEG = Join-Path $PWD 'target/video-tools/imageio_ffmpeg/binaries/ffmpeg-win-x86_64-v9.0.2.exe'
$shots = @(
    @{ Name='space'; Args=@('--record-tour','target/trailer/space.mp4') },
    @{ Name='mercury'; Args=@('--record-video','target/trailer/mercury.mp4','--mercury','--frames','540') },
    @{ Name='venus'; Args=@('--record-video','target/trailer/venus.mp4','--venus','--frames','540') },
    @{ Name='brasa'; Args=@('--record-video','target/trailer/brasa.mp4','--celebration-demo','--frames','150') },
    @{ Name='lirio'; Args=@('--record-video','target/trailer/lirio.mp4','--celebration-demo','--venus','--frames','150') },
    @{ Name='crew'; Args=@('--record-video','target/trailer/crew.mp4','--crew-preview','--frames','240') }
)
foreach ($shot in $shots) {
    Write-Output ('Grabando: ' + $shot.Name)
    & './target/release/rana-solar.exe' @($shot.Args) --cinematic
    if ($LASTEXITCODE -ne 0) { throw ('Error en toma ' + $shot.Name) }
}
