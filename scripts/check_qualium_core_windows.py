import ctypes
from ctypes import wintypes
import subprocess

user32 = ctypes.windll.user32

# Get all qualium-core PIDs
out = subprocess.check_output(['powershell', '-Command', '(Get-Process qualium-core -ErrorAction SilentlyContinue).Id']).decode().strip()
pids = [int(p.strip()) for p in out.splitlines() if p.strip()]
print('All qualium-core PIDs:', pids)

def enum_windows_callback(hwnd, extra):
    pid = wintypes.DWORD()
    user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
    if pid.value in pids:
        visible = user32.IsWindowVisible(hwnd)
        length = user32.GetWindowTextLengthW(hwnd)
        buff = ctypes.create_unicode_buffer(length + 1)
        user32.GetWindowTextW(hwnd, buff, length + 1)
        title = buff.value
        rect = wintypes.RECT()
        user32.GetWindowRect(hwnd, ctypes.byref(rect))
        w = rect.right - rect.left
        h = rect.bottom - rect.top
        print(f'PID {pid.value} HWND {hwnd}: visible={visible} title="{title}" size=({w}x{h})')
    return True

WNDENUMPROC = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)
user32.EnumWindows(WNDENUMPROC(enum_windows_callback), 0)
