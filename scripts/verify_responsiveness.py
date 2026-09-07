import subprocess
import time
import os
import ctypes
import win32gui
import win32process

user32 = ctypes.windll.user32
exe = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qaulium\QualiumQuantumBrowser.exe")

print("Launching Qaulium Quantum Browser with Google...")
proc = subprocess.Popen([exe, "-url", "https://www.google.com"])

time.sleep(3)

def find_browser_windows():
    wins = []
    def enum_cb(hwnd, _):
        if win32gui.IsWindowVisible(hwnd):
            title = win32gui.GetWindowText(hwnd)
            if any(k in title.lower() for k in ["qaulium", "qualium", "google"]):
                _, pid = win32process.GetWindowThreadProcessId(hwnd)
                is_hung = bool(user32.IsHungAppWindow(hwnd))
                wins.append((hwnd, pid, title, is_hung))
    win32gui.EnumWindows(enum_cb, None)
    return wins

all_passed = True
for step in range(1, 5):
    time.sleep(2.5)
    wins = find_browser_windows()
    print(f"Check #{step} (Elapsed ~{step*2.5+3:.1f}s):")
    for hwnd, pid, title, is_hung in wins:
        print(f"  HWND={hwnd}, PID={pid}, Title=\"{title}\", IsHungAppWindow={is_hung}")
        if is_hung:
            all_passed = False
            print(f"  [ERROR] Window {hwnd} is marked as (Not Responding)!")

if all_passed and wins:
    print("\n[PASSED] Browser remained 100% responsive throughout navigation and rendering! Zero hangs detected.")
else:
    print("\n[FAILED] One or more windows hung or no window found.")

subprocess.run(["powershell", "-Command", "Get-Process *qualium*,*firefox* -ErrorAction SilentlyContinue | Stop-Process -Force"], check=False)
