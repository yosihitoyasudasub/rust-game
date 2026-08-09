# Minimal static server for local WASM testing.
# .wasm must be served as application/wasm or instantiateStreaming refuses it.
param([int]$Port = 8123)

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$listener = New-Object System.Net.HttpListener
$listener.Prefixes.Add("http://localhost:$Port/")
$listener.Start()
Write-Host "serving $root on http://localhost:$Port/"

$mime = @{
    '.html' = 'text/html; charset=utf-8'
    '.js'   = 'application/javascript'
    '.wasm' = 'application/wasm'
    '.ttf'  = 'font/ttf'
    '.otf'  = 'font/otf'
}

while ($listener.IsListening) {
    $ctx = $listener.GetContext()
    $rel = [System.Uri]::UnescapeDataString($ctx.Request.Url.AbsolutePath).TrimStart('/')
    if ($rel -eq '') { $rel = 'index.html' }
    $path = Join-Path $root $rel

    if (Test-Path $path -PathType Leaf) {
        $bytes = [System.IO.File]::ReadAllBytes($path)
        $ext = [System.IO.Path]::GetExtension($path).ToLower()
        $ctx.Response.ContentType = if ($mime.ContainsKey($ext)) { $mime[$ext] } else { 'application/octet-stream' }
        $ctx.Response.ContentLength64 = $bytes.Length
        $ctx.Response.OutputStream.Write($bytes, 0, $bytes.Length)
        Write-Host "200 $rel ($($bytes.Length))"
        Add-Content -Path (Join-Path $root 'access.log') -Value "200 $rel ($($bytes.Length))"
    } else {
        $ctx.Response.StatusCode = 404
        Write-Host "404 $rel"
        Add-Content -Path (Join-Path $root 'access.log') -Value "404 $rel"
    }
    $ctx.Response.Close()
}
