[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [ValidateSet('tsxui', 'rustui')]
  [string]$Ui
)

$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;

[StructLayout(LayoutKind.Sequential)]
public struct WindowPoint { public int X; public int Y; }
[StructLayout(LayoutKind.Sequential)]
public struct WindowRect { public int Left; public int Top; public int Right; public int Bottom; }
[StructLayout(LayoutKind.Sequential)]
public struct WindowMonitorInfo { public int Size; public WindowRect Monitor; public WindowRect Work; public uint Flags; }
[StructLayout(LayoutKind.Sequential)]
public struct WindowMinMaxInfo {
  public WindowPoint Reserved;
  public WindowPoint MaxSize;
  public WindowPoint MaxPosition;
  public WindowPoint MinTrackSize;
  public WindowPoint MaxTrackSize;
}

public static class NativeWindowConstraintsProbe {
  public delegate bool EnumWindowsCallback(IntPtr hwnd, IntPtr parameter);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumWindowsCallback callback, IntPtr parameter);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassNameW(IntPtr hwnd, StringBuilder className, int length);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out WindowRect rect);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hwnd, out WindowRect rect);
  [DllImport("user32.dll")] public static extern IntPtr MonitorFromWindow(IntPtr hwnd, uint flags);
  [DllImport("user32.dll", EntryPoint = "GetMonitorInfoW")] public static extern bool GetMonitorInfo(IntPtr monitor, ref WindowMonitorInfo info);
  [DllImport("user32.dll", EntryPoint = "SendMessageW")] static extern IntPtr SendMessage(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll", EntryPoint = "SendMessageW")] static extern IntPtr SendIconMessage(IntPtr hwnd, uint message, UIntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
  [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] static extern IntPtr LoadLibraryExW(string file, IntPtr reserved, uint flags);
  [DllImport("kernel32.dll", EntryPoint = "FindResourceW", CharSet = CharSet.Unicode, SetLastError = true)] static extern IntPtr FindResource(IntPtr module, IntPtr name, IntPtr type);
  [DllImport("kernel32.dll")] static extern bool FreeLibrary(IntPtr module);

  public static bool HasWindowIcons(IntPtr hwnd) {
    return SendIconMessage(hwnd, 0x007F, new UIntPtr(1), IntPtr.Zero) != IntPtr.Zero
      && SendIconMessage(hwnd, 0x007F, UIntPtr.Zero, IntPtr.Zero) != IntPtr.Zero;
  }

  public static bool HasEmbeddedIcon(string file) {
    var module = LoadLibraryExW(file, IntPtr.Zero, 2); // LOAD_LIBRARY_AS_DATAFILE
    if (module == IntPtr.Zero) return false;
    try { return FindResource(module, new IntPtr(1), new IntPtr(14)) != IntPtr.Zero; }
    finally { FreeLibrary(module); }
  }

  public static IntPtr FindWindow(int processId) {
    IntPtr found = IntPtr.Zero;
    EnumWindows(delegate(IntPtr hwnd, IntPtr parameter) {
      uint owner;
      GetWindowThreadProcessId(hwnd, out owner);
      var className = new StringBuilder(128);
      GetClassNameW(hwnd, className, className.Capacity);
      if (owner == processId && className.ToString() == "NshWindow" && IsWindowVisible(hwnd)) {
        found = hwnd;
        return false;
      }
      return true;
    }, IntPtr.Zero);
    return found;
  }

  public static WindowMinMaxInfo ReadMinimum(IntPtr hwnd) {
    var info = new WindowMinMaxInfo();
    var pointer = Marshal.AllocHGlobal(Marshal.SizeOf<WindowMinMaxInfo>());
    try {
      Marshal.StructureToPtr(info, pointer, false);
      SendMessage(hwnd, 0x0024, IntPtr.Zero, pointer); // WM_GETMINMAXINFO
      return Marshal.PtrToStructure<WindowMinMaxInfo>(pointer);
    } finally {
      Marshal.FreeHGlobal(pointer);
    }
  }
}
'@

$appDirectory = if ($Ui -eq 'rustui') { '.nsh/out/rustui/Zeedle' } else { '.nsh/out/Zeedle' }
$executable = Join-Path $appDirectory 'Zeedle.exe'
if (!(Test-Path -LiteralPath $executable)) { throw "Missing packaged executable: $executable" }

$process = Start-Process -FilePath (Resolve-Path -LiteralPath $executable).Path -PassThru
try {
  $deadline = [DateTime]::UtcNow.AddSeconds(20)
  $window = [IntPtr]::Zero
  do {
    $process.Refresh()
    $window = [NativeWindowConstraintsProbe]::FindWindow($process.Id)
    if ($process.HasExited) { throw "Packaged $Ui UI exited with code $($process.ExitCode)" }
    if ($window -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 100 }
  } while ($window -eq [IntPtr]::Zero -and [DateTime]::UtcNow -lt $deadline)
  if ($window -eq [IntPtr]::Zero) { throw "Packaged $Ui UI did not create a visible NSH window" }
  $firstVisible = [WindowRect]::new()
  if (![NativeWindowConstraintsProbe]::GetWindowRect($window, [ref]$firstVisible)) { throw 'Initial GetWindowRect failed' }
  if ($Ui -eq 'rustui') { Start-Sleep -Seconds 2 }

  $client = [WindowRect]::new()
  $outer = [WindowRect]::new()
  $monitorInfo = [WindowMonitorInfo]::new()
  $monitorInfo.Size = [Runtime.InteropServices.Marshal]::SizeOf($monitorInfo)
  $monitor = [NativeWindowConstraintsProbe]::MonitorFromWindow($window, 2)
  if (![NativeWindowConstraintsProbe]::GetMonitorInfo($monitor, [ref]$monitorInfo)) { throw 'GetMonitorInfo failed' }
  if (![NativeWindowConstraintsProbe]::GetWindowRect($window, [ref]$outer)) { throw 'GetWindowRect failed' }
  if (![NativeWindowConstraintsProbe]::GetClientRect($window, [ref]$client)) { throw 'GetClientRect failed' }
  $dpi = [NativeWindowConstraintsProbe]::GetDpiForWindow($window)
  $clientWidth = [Math]::Round(($client.Right - $client.Left) * 96.0 / $dpi)
  $clientHeight = [Math]::Round(($client.Bottom - $client.Top) * 96.0 / $dpi)
  $minimum = [NativeWindowConstraintsProbe]::ReadMinimum($window)
  $frameWidth = ($outer.Right - $outer.Left) - ($client.Right - $client.Left)
  $frameHeight = ($outer.Bottom - $outer.Top) - ($client.Bottom - $client.Top)
  $minimumWidth = [Math]::Round(($minimum.MinTrackSize.X - $frameWidth) * 96.0 / $dpi)
  $minimumHeight = [Math]::Round(($minimum.MinTrackSize.Y - $frameHeight) * 96.0 / $dpi)
  $centerErrorX = [Math]::Abs((($outer.Left + $outer.Right) - ($monitorInfo.Work.Left + $monitorInfo.Work.Right)) / 2)
  $centerErrorY = [Math]::Abs((($outer.Top + $outer.Bottom) - ($monitorInfo.Work.Top + $monitorInfo.Work.Bottom)) / 2)
  $startupGeometryChanged = @(
    [Math]::Abs($firstVisible.Left - $outer.Left),
    [Math]::Abs($firstVisible.Top - $outer.Top),
    [Math]::Abs($firstVisible.Right - $outer.Right),
    [Math]::Abs($firstVisible.Bottom - $outer.Bottom)
  ) | Where-Object { $_ -gt 1 }
  Write-Output "${Ui}: first rect $($firstVisible.Left),$($firstVisible.Top)-$($firstVisible.Right),$($firstVisible.Bottom); final rect $($outer.Left),$($outer.Top)-$($outer.Right),$($outer.Bottom); client ${clientWidth}x${clientHeight}, minimum ${minimumWidth}x${minimumHeight} points, center error ${centerErrorX}x${centerErrorY}px at DPI $dpi"
  if ($centerErrorX -gt 2 -or $centerErrorY -gt 2) {
    throw "$Ui startup window should be centered in the monitor work area"
  }
  if ($startupGeometryChanged) {
    throw "$Ui window moved or resized after its first visible frame"
  }
  $hasWindowIcons = [NativeWindowConstraintsProbe]::HasWindowIcons($window)
  $hasEmbeddedIcon = [NativeWindowConstraintsProbe]::HasEmbeddedIcon((Resolve-Path -LiteralPath $executable).Path)
  Write-Output "${Ui}: window icons $hasWindowIcons, embedded executable icon $hasEmbeddedIcon"
  if (!$hasWindowIcons -or !$hasEmbeddedIcon) {
    throw "$Ui must expose large/small taskbar icons and embed an executable icon"
  }
  if ([Math]::Abs($clientWidth - 900) -gt 1 -or [Math]::Abs($clientHeight - 510) -gt 1) {
    throw "$Ui startup client size should be 900x510 points"
  }
  if ([Math]::Abs($minimumWidth - 850) -gt 1 -or [Math]::Abs($minimumHeight - 500) -gt 1) {
    throw "$Ui native minimum should be 850x500 content points"
  }
} finally {
  if ($window -ne [IntPtr]::Zero) { [void][NativeWindowConstraintsProbe]::PostMessageW($window, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) }
  if (!$process.HasExited -and !$process.WaitForExit(5000)) { Stop-Process -Id $process.Id -Force }
}
