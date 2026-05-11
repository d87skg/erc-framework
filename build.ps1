# ERC 纯净构建脚本
$env:CARGO_INCREMENTAL = "0"
Remove-Item -Recurse -Force target -ErrorAction SilentlyContinue
cargo build --release --locked -p erc-gateway
cargo build --release --locked -p erc-query-api
Write-Host "✅ Build complete"
