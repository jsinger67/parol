param(
    [ValidatePattern("debug|release")]
    $Config = "debug"
)

Write-Host "Please call" -ForegroundColor Green
Write-Host "  cargo make --task generate_examples" -ForegroundColor Yellow
Write-Host "in workspace's root to generate the examples." -ForegroundColor Green
