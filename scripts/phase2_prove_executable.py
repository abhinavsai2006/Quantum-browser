import os, sys, time, subprocess, ctypes
from ctypes import wintypes

user32 = ctypes.windll.user32

print("======================================================================")
print("PHASE 2: PROVE THE CURRENT EXECUTABLE — PROCESS TREE & WINDOW RECORD")
print("======================================================================")

exe_path = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium\QualiumQuantumBrowser.exe")

print(f"Target Executable: {exe_path}")

# Kill stale processes first
subprocess.run(["powershell", "-Command", "Get-Process QualiumQuantumBrowser, qualium-daemon, qualium-core, firefox -ErrorAction SilentlyContinue | Stop-Process -Force"], capture_output=True)
time.sleep(1)

# Clean locks across both Qualium and Qaulium profile folders
for p_name in ["Qualium", "Qaulium"]:
    prof_dir = os.path.expandvars(rf"%LOCALAPPDATA%\{p_name}\Profile")
    for lock in ["parent.lock", ".parentlock"]:
        lp = os.path.join(prof_dir, lock)
        if os.path.exists(lp):
            try: os.remove(lp)
            except: pass

target_url = sys.argv[1] if len(sys.argv) > 1 else None
out_img = sys.argv[2] if len(sys.argv) > 2 else r"C:\Users\mndab\.gemini\antigravity-ide\brain\38ef44a2-12b9-4657-8f35-3220c9d8b70b\qualium_v5_live_window5.png"

cmd = [exe_path]
if target_url:
    cmd.extend(["-url", target_url])

# Start QualiumQuantumBrowser.exe
proc = subprocess.Popen(cmd)
print(f"Spawned QualiumQuantumBrowser.exe PID: {proc.pid} with cmd: {cmd}")
time.sleep(7)

# Query processes using PowerShell WMI
ps_cmd = """
Get-CimInstance Win32_Process | Where-Object { $_.Name -match 'Qualium|qualium|firefox' } | 
Select-Object ProcessId, ParentProcessId, Name, CommandLine, ExecutablePath |
Format-List
"""
proc_info = subprocess.check_output(["powershell", "-Command", ps_cmd]).decode("utf-8", "ignore")
print("\n=== Active Process Tree & Command Lines ===")
print(proc_info.strip())

# Window inspection
print("\n=== Window Inspection ===")
def enum_cb(hwnd, extra):
    if user32.IsWindowVisible(hwnd):
        length = user32.GetWindowTextLengthW(hwnd)
        buff = ctypes.create_unicode_buffer(length + 1)
        user32.GetWindowTextW(hwnd, buff, length + 1)
        title = buff.value
        cls_name = ctypes.create_unicode_buffer(256)
        user32.GetClassNameW(hwnd, cls_name, 256)
        pid = wintypes.DWORD()
        user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
        if any(term in title.lower() or term in cls_name.value.lower() for term in ["qualium", "mozilla", "firefox"]):
            rect = wintypes.RECT()
            user32.GetWindowRect(hwnd, ctypes.byref(rect))
            w = rect.right - rect.left
            h = rect.bottom - rect.top
            print(f"PID {pid.value} HWND {hwnd}: class='{cls_name.value}' title='{title}' size=({w}x{h})")
            if w > 400 and h > 300:
                try:
                    subprocess.run(["py", "-3", "scripts/capture_window.py", out_img], capture_output=True)
                    print(f"Captured live screenshot to {out_img}")
                except Exception as e:
                    print("Capture error:", e)
    return True

WNDENUMPROC = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)
user32.EnumWindows(WNDENUMPROC(enum_cb), 0)

# Keep process alive for 3 more seconds before termination
time.sleep(3)

# Terminate test process
subprocess.run(["powershell", "-Command", "Get-Process QualiumQuantumBrowser, qualium-daemon, qualium-core, firefox -ErrorAction SilentlyContinue | Stop-Process -Force"], capture_output=True)
time.sleep(1)
print("======================================================================")
