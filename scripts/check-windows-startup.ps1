Add-Type -AssemblyName System.Drawing
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Drawing;
public class StartupProbe {
  [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
  [DllImport("dwmapi.dll")] static extern int DwmGetWindowAttribute(IntPtr h, uint a, out int value, uint size);
  public static bool Presented(IntPtr h) { int cloak; Rect r; GetWindowRect(h,out r); DwmGetWindowAttribute(h,14,out cloak,4); return IsWindowVisible(h) && cloak == 0 && r.Right > -30000 && r.Bottom > -30000; }
  delegate bool Callback(IntPtr h, IntPtr p);
  [DllImport("user32.dll")] static extern bool EnumWindows(Callback cb, IntPtr p);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint id);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr h, System.Text.StringBuilder name, int length);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
  [StructLayout(LayoutKind.Sequential)] public struct Info { public int Size; public Rect Monitor, Work; public uint Flags; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out Rect r);
  [DllImport("user32.dll")] public static extern IntPtr MonitorFromWindow(IntPtr h, uint f);
  [DllImport("user32.dll")] public static extern bool GetMonitorInfo(IntPtr h, ref Info i);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll", EntryPoint="SendMessageW")] static extern IntPtr SendMessageIcon(IntPtr h, uint m, UIntPtr w, IntPtr l);
  public static bool HasIcons(IntPtr h) { return SendMessageIcon(h,127,new UIntPtr(1),IntPtr.Zero)!=IntPtr.Zero && SendMessageIcon(h,127,UIntPtr.Zero,IntPtr.Zero)!=IntPtr.Zero; }
  [DllImport("user32.dll")] static extern bool PrintWindow(IntPtr h, IntPtr dc, uint flags);
  public static IntPtr Find(int id) {
    IntPtr found = IntPtr.Zero;
    EnumWindows(delegate(IntPtr h, IntPtr p) { uint pid; GetWindowThreadProcessId(h, out pid); var name = new System.Text.StringBuilder(128); GetClassName(h,name,128); if (pid == id && name.ToString() == "NshWindow") { found=h; return false; } return true; }, IntPtr.Zero);
    return found;
  }
  public static void Capture(IntPtr h, string path) {
    Rect r; GetWindowRect(h, out r);
    using(var bmp = new Bitmap(r.Right-r.Left, r.Bottom-r.Top)) using(var g=Graphics.FromImage(bmp)) {
      var dc=g.GetHdc(); try { PrintWindow(h,dc,2); } finally { g.ReleaseHdc(dc); }
      bmp.Save(path);
    }
  }
}
'@
$appPath = (Resolve-Path .nsh/out/Zeedle/Zeedle.exe).Path
[void][StartupProbe]::SetThreadDpiAwarenessContext([IntPtr](-4))
$launched = [Diagnostics.Process]::Start($appPath)
$timer = [Diagnostics.Stopwatch]::StartNew()
$hidden = 0
$visible = $false
$handle = [IntPtr]::Zero
try {
  while ($timer.ElapsedMilliseconds -lt 15000) {
    $handle = [StartupProbe]::Find($launched.Id)
    if ($handle -ne [IntPtr]::Zero) {
      if ([StartupProbe]::Presented($handle)) { $visible = $true; break }
      $hidden++
    }
    Start-Sleep -Milliseconds 10
  }
  if (!$visible) { throw 'Packaged window never became visible' }
  $rect = New-Object StartupProbe+Rect
  $info = New-Object StartupProbe+Info
  $info.Size = [Runtime.InteropServices.Marshal]::SizeOf($info)
  [void][StartupProbe]::GetWindowRect($handle, [ref]$rect)
  [void][StartupProbe]::GetMonitorInfo([StartupProbe]::MonitorFromWindow($handle,2), [ref]$info)
  $dx = [Math]::Abs(($rect.Left+$rect.Right)-($info.Work.Left+$info.Work.Right))/2
  $dy = [Math]::Abs(($rect.Top+$rect.Bottom)-($info.Work.Top+$info.Work.Bottom))/2
  [StartupProbe]::Capture($handle, (Join-Path (Get-Location) 'target/windows-first-visible.png'))
  Start-Sleep -Milliseconds 150
  [StartupProbe]::Capture($handle, (Join-Path (Get-Location) 'target/windows-packaged-startup.png'))
  $hasIcons = [StartupProbe]::HasIcons($handle)
  [PSCustomObject]@{ HiddenSamples=$hidden; VisibleAfterMs=$timer.ElapsedMilliseconds; CenterErrorX=$dx; CenterErrorY=$dy; TaskbarIcons=$hasIcons; Width=($rect.Right-$rect.Left); Height=($rect.Bottom-$rect.Top); Left=$rect.Left; Top=$rect.Top; WorkLeft=$info.Work.Left; WorkTop=$info.Work.Top; WorkRight=$info.Work.Right; WorkBottom=$info.Work.Bottom } | Format-List
  if ($hidden -eq 0 -or $dx -gt 1 -or $dy -gt 1 -or !$hasIcons) { throw 'Startup visibility, centering, or taskbar icon check failed' }
} finally {
  if ($handle -ne [IntPtr]::Zero) { [void][StartupProbe]::PostMessage($handle,16,[IntPtr]::Zero,[IntPtr]::Zero) }
  [void]$launched.WaitForExit(5000)
}
