import os, sys, time, subprocess, ctypes
from ctypes import wintypes

user32 = ctypes.windll.user32
gdi32 = ctypes.windll.gdi32

PW_RENDERFULLCONTENT = 0x00000002

def capture_window_screenshot(output_path, timeout=15):
    target_hwnd = None
    WNDENUMPROC = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)

    for sec in range(timeout):
        def enum_cb(hwnd, extra):
            nonlocal target_hwnd
            if user32.IsWindowVisible(hwnd):
                length = user32.GetWindowTextLengthW(hwnd)
                buff = ctypes.create_unicode_buffer(length + 1)
                user32.GetWindowTextW(hwnd, buff, length + 1)
                title = buff.value
                cls_name = ctypes.create_unicode_buffer(256)
                user32.GetClassNameW(hwnd, cls_name, 256)
                rect = wintypes.RECT()
                user32.GetWindowRect(hwnd, ctypes.byref(rect))
                w = rect.right - rect.left
                h = rect.bottom - rect.top
                if (("qualium" in title.lower() or "qaulium" in title.lower()) or cls_name.value == "MozillaWindowClass") and w > 200 and h > 20:
                    target_hwnd = hwnd
                    return False
            return True

        user32.EnumWindows(WNDENUMPROC(enum_cb), 0)
        if target_hwnd:
            break
        time.sleep(1)

    if not target_hwnd:
        print("No Qualium window found to capture after polling.")
        return False

    print(f"Found Qualium window HWND: {target_hwnd}")

    user32.ShowWindow(target_hwnd, 9) # SW_RESTORE
    user32.SetWindowPos(target_hwnd, 0, 50, 50, 1280, 820, 0x0040)
    time.sleep(1.5)

    rect = wintypes.RECT()
    user32.GetWindowRect(target_hwnd, ctypes.byref(rect))
    w = max(1, rect.right - rect.left)
    h = max(1, rect.bottom - rect.top)
    print(f"Window dimensions: {w}x{h}")

    # Create compatible DC & bitmap
    hwnd_dc = user32.GetWindowDC(target_hwnd)
    mem_dc = gdi32.CreateCompatibleDC(hwnd_dc)
    bitmap = gdi32.CreateCompatibleBitmap(hwnd_dc, w, h)
    old_bmp = gdi32.SelectObject(mem_dc, bitmap)

    # Call PrintWindow with PW_RENDERFULLCONTENT
    result = user32.PrintWindow(target_hwnd, mem_dc, PW_RENDERFULLCONTENT)
    if not result:
        # Fallback to standard PrintWindow
        result = user32.PrintWindow(target_hwnd, mem_dc, 0)
    print(f"PrintWindow result: {result}")

    # Use PIL to read the bitmap data if available, or write standard BMP then convert to PNG
    # Standard BMP header
    file_header_size = 14
    info_header_size = 40
    image_size = w * h * 4
    total_size = file_header_size + info_header_size + image_size

    import struct
    bmp_header = struct.pack(
        '<2sIHHI',
        b'BM',
        total_size,
        0, 0,
        file_header_size + info_header_size
    )
    dib_header = struct.pack(
        '<IiiHHIIIIII',
        info_header_size,
        w, -h, # top-down
        1, 32, # 32 bpp
        0, image_size,
        0, 0, 0, 0
    )

    buf = ctypes.create_string_buffer(image_size)
    gdi32.GetBitmapBits(bitmap, image_size, buf)

    bmp_path = output_path.replace(".png", ".bmp")
    with open(bmp_path, "wb") as f:
        f.write(bmp_header)
        f.write(dib_header)
        f.write(buf.raw)

    gdi32.SelectObject(mem_dc, old_bmp)
    gdi32.DeleteObject(bitmap)
    gdi32.DeleteDC(mem_dc)
    user32.ReleaseDC(target_hwnd, hwnd_dc)

    # Convert BMP to PNG using powershell System.Drawing
    ps_cmd = f"""
    Add-Type -AssemblyName System.Drawing
    $img = [System.Drawing.Image]::FromFile('{bmp_path}')
    $img.Save('{output_path}', [System.Drawing.Imaging.ImageFormat]::Png)
    $img.Dispose()
    Remove-Item '{bmp_path}' -Force -ErrorAction SilentlyContinue
    """
    subprocess.run(["powershell", "-Command", ps_cmd], capture_output=True)

    if os.path.exists(output_path) and os.path.getsize(output_path) > 5000:
        print(f"Captured {output_path} ({os.path.getsize(output_path)} bytes)")
        return True
    else:
        print(f"Captured file small or missing: {os.path.getsize(output_path) if os.path.exists(output_path) else 'N/A'}")
        return False

if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "qualium_window_test.png"
    capture_window_screenshot(out)
