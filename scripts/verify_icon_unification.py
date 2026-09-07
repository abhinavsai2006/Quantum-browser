import os
import sys
import time
import subprocess
import ctypes
from ctypes import wintypes
import win32gui
import win32ui
import win32con
from PIL import Image, ImageGrab
import numpy as np

ARTIFACT_DIR = r"C:\Users\mndab\.gemini\antigravity-ide\brain\b0a47078-44ea-43d7-ae34-0240dd9ce43a"

def verify_ico_transparency(ico_path):
    print(f"\n[Check] Validating ICO transparency: {ico_path}")
    im = Image.open(ico_path)
    im = im.convert("RGBA")
    arr = np.array(im)
    corners = [arr[0, 0], arr[0, -1], arr[-1, 0], arr[-1, -1]]
    print(f"  Corners alpha: {[c[3] for c in corners]}")
    assert all(c[3] == 0 for c in corners), "Corners must be fully transparent (alpha=0), no white boxes!"
    print("  [PASS] Zero white corners, proper RGBA alpha transparency confirmed.")

def verify_extracted_pe_icon(exe_path, out_png):
    print(f"\n[Check] Extracting and validating PE icon from: {exe_path}")
    large, small = win32gui.ExtractIconEx(exe_path, 0)
    if not large:
        raise RuntimeError(f"No icon found in {exe_path}")
    hicon = large[0]
    hdc = win32ui.CreateDCFromHandle(win32gui.GetDC(0))
    hbmp = win32ui.CreateBitmap()
    hbmp.CreateCompatibleBitmap(hdc, 32, 32)
    memdc = hdc.CreateCompatibleDC()
    memdc.SelectObject(hbmp)
    win32gui.DrawIconEx(memdc.GetSafeHdc(), 0, 0, hicon, 32, 32, 0, 0, win32con.DI_NORMAL)
    bmpinfo = hbmp.GetInfo()
    bmpstr = hbmp.GetBitmapBits(True)
    img = Image.frombuffer('RGBA', (bmpinfo['bmWidth'], bmpinfo['bmHeight']), bmpstr, 'raw', 'BGRA', 0, 1)
    img.save(out_png)
    win32gui.DestroyIcon(hicon)
    for h in small: win32gui.DestroyIcon(h)
    for h in large[1:]: win32gui.DestroyIcon(h)

    arr = np.array(img)
    corners = [arr[0, 0], arr[0, -1], arr[-1, 0], arr[-1, -1]]
    print(f"  Extracted icon saved to {out_png}")
    print(f"  Corners alpha: {[c[3] for c in corners]}")
    assert all(c[3] == 0 for c in corners), "PE icon corners must be transparent!"
    print("  [PASS] PE icon extracted cleanly with transparent corners.")

def verify_visual_elements():
    print("\n[Check] Validating VisualElements PNGs...")
    ve_150 = os.path.join(r"e:\Qaulium AI\Broswer", "runtime", "browser", "VisualElements", "VisualElements_150.png")
    im = Image.open(ve_150)
    arr = np.array(im)
    corners = [arr[0, 0], arr[0, -1], arr[-1, 0], arr[-1, -1]]
    print(f"  VisualElements_150 corners alpha: {[c[3] for c in corners]}")
    assert all(c[3] == 0 for c in corners), "VisualElements corners must be transparent!"
    print("  [PASS] VisualElements PNGs verified.")

def main():
    print("=== QUALIUM BROWSER ICON & BRANDING UNIFICATION VERIFICATION ===")
    local_ico = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium\resources\qualium.ico")
    verify_ico_transparency(local_ico)

    local_core_exe = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium\runtime\qualium-core.exe")
    pe_icon_png = os.path.join(ARTIFACT_DIR, "verified_extracted_core_icon.png")
    verify_extracted_pe_icon(local_core_exe, pe_icon_png)

    verify_visual_elements()
    print("\n[SUCCESS] All icon checks passed 100%!")

if __name__ == '__main__':
    main()
