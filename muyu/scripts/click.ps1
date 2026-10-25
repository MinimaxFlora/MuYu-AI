param(
  [int]$X,
  [int]$Y
)

Add-Type @"
using System;
using System.Runtime.InteropServices;
public class MClick {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
}
"@

$p = Get-Process muyu -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if ($p) { [MClick]::SetForegroundWindow($p.MainWindowHandle) | Out-Null }
Start-Sleep -Milliseconds 250
[MClick]::SetCursorPos($X, $Y) | Out-Null
Start-Sleep -Milliseconds 120
[MClick]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
[MClick]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
Write-Output "clicked $X,$Y"
