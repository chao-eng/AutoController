# select_region.ps1
# Self-contained WinForms Screen Region Selector for AutoController
# Supports multi-monitor virtual screen setups

Add-Type -AssemblyName System.Windows.Forms, System.Drawing

$bounds = [System.Windows.Forms.SystemInformation]::VirtualScreen

$form = New-Object System.Windows.Forms.Form
$form.Text = 'AutoController Region Selector'
$form.FormBorderStyle = 'None'
$form.StartPosition = 'Manual'
$form.Location = New-Object System.Drawing.Point $bounds.Left, $bounds.Top
$form.Size = New-Object System.Drawing.Size $bounds.Width, $bounds.Height
$form.TopMost = $true
$form.Opacity = 0.35
$form.BackColor = 'Black'
$form.Cursor = [System.Windows.Forms.Cursors]::Cross
$form.KeyPreview = $true
$form.ShowInTaskbar = $false

# Double buffering to prevent flickering
$form.GetType().GetMethod('SetStyle', [System.Reflection.BindingFlags]::NonPublic -bor [System.Reflection.BindingFlags]::Instance).Invoke($form, @([System.Windows.Forms.ControlStyles]::OptimizedDoubleBuffer -bor [System.Windows.Forms.ControlStyles]::AllPaintingInWmPaint -bor [System.Windows.Forms.ControlStyles]::UserPaint, $true))

$tip = New-Object System.Windows.Forms.Label
$tip.AutoSize = $true
$tip.ForeColor = [System.Drawing.Color]::FromArgb(34, 197, 94) # Neon green
$tip.BackColor = [System.Drawing.Color]::Transparent
$tip.Font = New-Object System.Drawing.Font 'Microsoft YaHei', 20, ([System.Drawing.FontStyle]::Bold)
$tip.Text = "🎯 鼠标左键点击并拖拽以框选标定 OCR 默认识别区 (按 ESC 取消)"
$tip.Location = New-Object System.Drawing.Point 80, 80
$form.Controls.Add($tip)

$liveCoord = New-Object System.Windows.Forms.Label
$liveCoord.AutoSize = $true
$liveCoord.ForeColor = [System.Drawing.Color]::Yellow
$liveCoord.BackColor = [System.Drawing.Color]::Transparent
$liveCoord.Font = New-Object System.Drawing.Font 'Consolas', 18
$liveCoord.Location = New-Object System.Drawing.Point 80, 140
$form.Controls.Add($liveCoord)

$script:startPt = $null
$script:rect = New-Object System.Drawing.Rectangle 0, 0, 0, 0
$script:result = $null

$form.add_MouseDown({
    param($s, $e)
    if ($e.Button -eq 'Left') {
        $script:startPt = $e.Location
        $script:rect = New-Object System.Drawing.Rectangle $e.X, $e.Y, 0, 0
    }
})

$form.add_MouseMove({
    param($s, $e)
    if ($script:startPt) {
        $x = [Math]::Min($script:startPt.X, $e.X)
        $y = [Math]::Min($script:startPt.Y, $e.Y)
        $w = [Math]::Abs($e.X - $script:startPt.X)
        $h = [Math]::Abs($e.Y - $script:startPt.Y)
        $script:rect = New-Object System.Drawing.Rectangle $x, $y, $w, $h
        $screenX = $bounds.Left + $x
        $screenY = $bounds.Top + $y
        $liveCoord.Text = "起始坐标: ($screenX, $screenY)  宽: $w px  高: $h px"
        $form.Invalidate()
    }
})

$form.add_MouseUp({
    param($s, $e)
    if ($script:startPt -and $script:rect.Width -gt 10 -and $script:rect.Height -gt 10) {
        $script:result = $script:rect
        $form.Close()
    }
    $script:startPt = $null
})

$form.add_Paint({
    param($s, $e)
    if ($script:rect.Width -gt 0 -and $script:rect.Height -gt 0) {
        # Neon green glowing dashed pen & semi-transparent brush
        $pen = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(34, 197, 94)), 3
        $pen.DashStyle = [System.Drawing.Drawing2D.DashStyle]::Dash
        $e.Graphics.DrawRectangle($pen, $script:rect)
        
        $brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(30, 34, 197, 94))
        $e.Graphics.FillRectangle($brush, $script:rect)
        $pen.Dispose(); $brush.Dispose()
    }
})

$form.add_KeyDown({
    param($s, $e)
    if ($e.KeyCode -eq 'Escape') { $script:result = $null; $form.Close() }
})

[void]$form.ShowDialog()
$form.Dispose()

if ($script:result) {
    $x = $bounds.Left + $script:result.X
    $y = $bounds.Top + $script:result.Y
    Write-Output "RESULT:$x,$y,$($script:result.Width),$($script:result.Height)"
} else {
    Write-Output "CANCELLED"
}
