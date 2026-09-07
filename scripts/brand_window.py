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

    target_ico = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium\resources\qualium.ico")
    if not os.path.exists(target_ico):
        target_ico = os.path.join(r"e:\Qaulium AI\Broswer", "resources", "qualium.ico")

    IMAGE_ICON = 1
    LR_LOADFROMFILE = 0x00000010
    hicon_big = user32.LoadImageW(0, target_ico, IMAGE_ICON, 32, 32, LR_LOADFROMFILE)
    hicon_small = user32.LoadImageW(0, target_ico, IMAGE_ICON, 16, 16, LR_LOADFROMFILE)

    for hwnd in found:
        user32.SetWindowTextW(hwnd, "Qualium Quantum Browser")
        if hicon_big:
            user32.SendMessageW(hwnd, WM_SETICON, ICON_BIG, hicon_big)
        if hicon_small:
            user32.SendMessageW(hwnd, WM_SETICON, ICON_SMALL, hicon_small)
        print(f"Branded HWND {hwnd} with title and Qualium Quantum Browser icon.")

if __name__ == "__main__":
    brand_windows()

