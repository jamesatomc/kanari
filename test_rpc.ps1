# Test RPC calls to Kanari node
$headers = @{"Content-Type" = "application/json"}

# Test getNodeInfo
Write-Host "Testing getNodeInfo..."
$body1 = '{"jsonrpc":"2.0","method":"kanari_getNodeInfo","params":[],"id":1}'
try {
    $response1 = Invoke-WebRequest -Uri "http://localhost:3031" -Method POST -Headers $headers -Body $body1
    Write-Host "Response: " $response1.Content
} catch {
    Write-Host "Error: " $_.Exception.Message
}

Write-Host ""

# Test getChainId
Write-Host "Testing getChainId..."
$body2 = '{"jsonrpc":"2.0","method":"kanari_getChainId","params":[],"id":2}'
try {
    $response2 = Invoke-WebRequest -Uri "http://localhost:3031" -Method POST -Headers $headers -Body $body2
    Write-Host "Response: " $response2.Content
} catch {
    Write-Host "Error: " $_.Exception.Message
}

Write-Host ""

# Test getBlockHeight
Write-Host "Testing getBlockHeight..."
$body3 = '{"jsonrpc":"2.0","method":"kanari_getBlockHeight","params":[],"id":3}'
try {
    $response3 = Invoke-WebRequest -Uri "http://localhost:3031" -Method POST -Headers $headers -Body $body3
    Write-Host "Response: " $response3.Content
} catch {
    Write-Host "Error: " $_.Exception.Message
}

Write-Host ""

# Test sendTransaction
Write-Host "Testing sendTransaction..."
$body4 = '{"jsonrpc":"2.0","method":"kanari_sendTransaction","params":[{"sender":"0x1234","recipient":"0x5678","amount":"1000","gas_limit":21000,"gas_price":1}],"id":4}'
try {
    $response4 = Invoke-WebRequest -Uri "http://localhost:3031" -Method POST -Headers $headers -Body $body4
    Write-Host "Response: " $response4.Content
} catch {
    Write-Host "Error: " $_.Exception.Message
}
