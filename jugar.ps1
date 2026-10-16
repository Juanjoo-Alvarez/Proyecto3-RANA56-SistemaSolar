param([switch]$SoloCompilar)
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $PSScriptRoot

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Instala Rust y Cargo antes de compilar.'
}
if (-not (Get-Command cmake -ErrorAction SilentlyContinue)) {
    $locator = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (Test-Path -LiteralPath $locator) {
        $installation = & $locator -latest -products '*' -property installationPath
        $cmakeFolder = Join-Path $installation 'Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin'
        if (Test-Path -LiteralPath (Join-Path $cmakeFolder 'cmake.exe')) {
            $env:PATH = $cmakeFolder + ';' + $env:PATH
        }
    }
}
if (-not $env:LIBCLANG_PATH) {
    $clangFolder = Join-Path $env:ProgramFiles 'LLVM\bin'
    if (Test-Path -LiteralPath (Join-Path $clangFolder 'libclang.dll')) {
        $env:LIBCLANG_PATH = $clangFolder
    }
}
if ($SoloCompilar) { cargo build --release --locked }
else { cargo run --release --locked }
exit $LASTEXITCODE
