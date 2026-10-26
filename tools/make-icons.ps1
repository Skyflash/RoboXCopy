# Generates every icon of the project from assets/icon.txt (the single source of the drawing).
#   assets/icon.ico        embedded in the exe (build.rs)
#   docs/favicon.ico       site favicon
#   docs/icon.svg          site favicon / logo (vector, crisp pixels)
#   docs/img/icon-180.png  apple-touch-icon
#   docs/img/icon-512.png  large version for previews
# Run from anywhere:  powershell -File tools\make-icons.ps1
param([string]$Root = (Split-Path $PSScriptRoot -Parent))

Add-Type -AssemblyName System.Drawing

# --- read the drawing -------------------------------------------------------------------------
$palette = @{}
$grid = @()
$inGrid = $false
foreach ($line in Get-Content "$Root\assets\icon.txt") {
    if ($line -like '#*') { continue }
    if ($line -eq '---') { $inGrid = $true; continue }
    if ($inGrid) { $grid += $line }
    elseif ($line -match '^(.)=([0-9A-Fa-f]{6})$') { $palette[[string]$Matches[1]] = $Matches[2].ToUpper() }
}
$n = $grid.Count
if ($n -eq 0 -or ($grid | Where-Object { $_.Length -ne $n })) { throw "icon.txt must be a square grid" }

function Get-Color([string]$ch) { [System.Drawing.ColorTranslator]::FromHtml('#' + $palette[$ch]) }

# --- render ----------------------------------------------------------------------------------
# Whole-number scales stay crisp (nearest neighbour). Other sizes are shrunk from the 256 px render.
function New-Render([int]$size) {
    $bmp = New-Object System.Drawing.Bitmap $size, $size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.Clear([System.Drawing.Color]::Transparent)
    if ($size % $n -eq 0) {
        $scale = $size / $n
        for ($r = 0; $r -lt $n; $r++) { for ($c = 0; $c -lt $n; $c++) {
            $ch = [string]$grid[$r][$c]
            if ($ch -ne '.') {
                $brush = New-Object System.Drawing.SolidBrush (Get-Color $ch)
                $g.FillRectangle($brush, $c * $scale, $r * $scale, $scale, $scale)
                $brush.Dispose()
            } } }
    } else {
        $big = New-Render 256
        $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $g.DrawImage($big, 0, 0, $size, $size)
        $big.Dispose()
    }
    $g.Dispose()
    return $bmp
}

function Get-PngBytes($bmp) {
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    return , $ms.ToArray()
}

function Write-Ico([string]$path, [int[]]$sizes) {
    $images = foreach ($s in $sizes) { $b = New-Render $s; , (Get-PngBytes $b); $b.Dispose() }
    $ms = New-Object System.IO.MemoryStream
    $w = New-Object System.IO.BinaryWriter $ms
    $w.Write([uint16]0); $w.Write([uint16]1); $w.Write([uint16]$sizes.Count)
    $offset = 6 + 16 * $sizes.Count
    for ($i = 0; $i -lt $sizes.Count; $i++) {
        $s = $sizes[$i]; $dim = if ($s -ge 256) { 0 } else { $s }
        $w.Write([byte]$dim); $w.Write([byte]$dim); $w.Write([byte]0); $w.Write([byte]0)
        $w.Write([uint16]1); $w.Write([uint16]32)
        $w.Write([uint32]$images[$i].Length); $w.Write([uint32]$offset)
        $offset += $images[$i].Length
    }
    foreach ($img in $images) { $w.Write([byte[]]$img) }
    $w.Flush()
    [System.IO.File]::WriteAllBytes($path, $ms.ToArray())
}

function Write-Png([string]$path, [int]$size) {
    $b = New-Render $size
    $b.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $b.Dispose()
}

function Write-Svg([string]$path) {
    # One <path> per colour, horizontal runs merged: small file, crisp at any size.
    $byColor = @{}
    for ($r = 0; $r -lt $n; $r++) {
        $c = 0
        while ($c -lt $n) {
            $ch = [string]$grid[$r][$c]
            if ($ch -eq '.') { $c++; continue }
            $start = $c
            while ($c -lt $n -and [string]$grid[$r][$c] -eq $ch) { $c++ }
            if (-not $byColor.ContainsKey($ch)) { $byColor[$ch] = New-Object System.Text.StringBuilder }
            [void]$byColor[$ch].Append("M$start $r" + "h$($c - $start)v1h-$($c - $start)z")
        }
    }
    $sb = New-Object System.Text.StringBuilder
    [void]$sb.AppendLine("<svg xmlns=`"http://www.w3.org/2000/svg`" viewBox=`"0 0 $n $n`" shape-rendering=`"crispEdges`">")
    [void]$sb.AppendLine('  <title>Robo-XCopy</title>')
    foreach ($key in ($byColor.Keys | Sort-Object)) {
        [void]$sb.AppendLine("  <path fill=`"#$($palette[$key])`" d=`"$($byColor[$key])`"/>")
    }
    [void]$sb.AppendLine('</svg>')
    [System.IO.File]::WriteAllText($path, $sb.ToString(), (New-Object System.Text.UTF8Encoding $false))
}

New-Item -ItemType Directory -Force "$Root\docs\img" | Out-Null
Write-Ico "$Root\assets\icon.ico" @(16, 24, 32, 48, 64, 128, 256)
Write-Ico "$Root\docs\favicon.ico" @(16, 32, 48)
Write-Svg "$Root\docs\icon.svg"
Write-Png "$Root\docs\img\icon-180.png" 180
Write-Png "$Root\docs\img\icon-512.png" 512
Write-Host "icons written from assets/icon.txt ($n x $n)"
