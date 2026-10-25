param(
  [string]$Out = "D:\AI 软件开发\AI\muyu\.shots\tauri.png"
)

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public struct RCAP { public int Left; public int Top; public int Right; public int Bottom; }
public struct PCAP { public int X; public int Y; }
public class MCap {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RCAP r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref PCAP p);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
}
"@

[MCap]::SetProcessDPIAware() | Out-Null

$p = Get-Process -ErrorAction SilentlyContinue |
  Where-Object { $_.MainWindowHandle -ne 0 -and ($_.ProcessName -eq "muyu" -or $_.ProcessName -like "MuYu*" -or $_.MainWindowTitle -match "^幕语") } |
  Select-Object -First 1
if (-not $p) { Write-Error "muyu window not found"; exit 1 }

[MCap]::ShowWindow($p.MainWindowHandle, 3) | Out-Null
[MCap]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
Start-Sleep -Milliseconds 700

$c = New-Object RCAP
[MCap]::GetClientRect($p.MainWindowHandle, [ref]$c) | Out-Null
$pt = New-Object PCAP
[MCap]::ClientToScreen($p.MainWindowHandle, [ref]$pt) | Out-Null

$bmp = New-Object System.Drawing.Bitmap $c.Right, $c.Bottom
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($pt.X, $pt.Y, 0, 0, $bmp.Size)
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()

Write-Output "captured $($c.Right)x$($c.Bottom) at $($pt.X),$($pt.Y) -> $Out"
