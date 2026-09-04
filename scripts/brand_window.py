import ctypes
from ctypes import wintypes
import time
import subprocess

user32 = ctypes.windll.user32

WM_SETICON = 0x0080
ICON_SMALL = 0
ICON_BIG = 1

def brand_windows():
    found = []
    def enum_cb(hwnd, extra):
        if user32.IsWindowVisible(hwnd):
            cls_name = ctypes.create_unicode_buffer(256)
            user32.GetClassNameW(hwnd, cls_name, 256)
            if cls_name.value == "MozillaWindowClass":
                found.append(hwnd)
        return True

    WNDENUMPROC = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)
    user32.EnumWindows(WNDENUMPROC(enum_cb), 0)

    for hwnd in found:
        user32.SetWindowTextW(hwnd, "Qualium Quantum Browser")
        print(f"Branded HWND {hwnd} to Qualium Quantum Browser")

if __name__ == "__main__":
    brand_windows()
