import os, sys, time, subprocess, ctypes
from ctypes import wintypes

user32 = ctypes.windll.user32
gdi32 = ctypes.windll.gdi32
PW_RENDERFULLCONTENT = 0x00000002

def capture_screenshot(output_path):
    target_hwnd = None
    def enum_cb(hwnd, extra):
        nonlocal target_hwnd
        if user32.IsWindowVisible(hwnd):
            cls_name = ctypes.create_unicode_buffer(256)
            user32.GetClassNameW(hwnd, cls_name, 256)
            rect = wintypes.RECT()
            user32.GetWindowRect(hwnd, ctypes.byref(rect))
            w = rect.right - rect.left
            h = rect.bottom - rect.top
            if cls_name.value == "MozillaWindowClass" and w > 400 and h > 300:
                target_hwnd = hwnd
                return False
        return True

    WNDENUMPROC = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)
    user32.EnumWindows(WNDENUMPROC(enum_cb), 0)

    if not target_hwnd:
        print(f"Window not found for {output_path}")
        return False

    user32.ShowWindow(target_hwnd, 9)
    user32.SetWindowPos(target_hwnd, 0, 50, 50, 1280, 820, 0x0040)
    time.sleep(1.2)

    rect = wintypes.RECT()
    user32.GetWindowRect(target_hwnd, ctypes.byref(rect))
    w = max(1, rect.right - rect.left)
    h = max(1, rect.bottom - rect.top)

    hwnd_dc = user32.GetWindowDC(target_hwnd)
    mem_dc = gdi32.CreateCompatibleDC(hwnd_dc)
    bitmap = gdi32.CreateCompatibleBitmap(hwnd_dc, w, h)
    old_bmp = gdi32.SelectObject(mem_dc, bitmap)

    user32.PrintWindow(target_hwnd, mem_dc, PW_RENDERFULLCONTENT)

    import zlib, struct
    bmi = struct.pack('<IIIHHIIIIII', 40, w, -h, 1, 32, 0, w * h * 4, 0, 0, 0, 0)
    raw_buffer = ctypes.create_string_buffer(w * h * 4)
    gdi32.GetDIBits(mem_dc, bitmap, 0, h, raw_buffer, bmi, 0)

    gdi32.SelectObject(mem_dc, old_bmp)
    gdi32.DeleteObject(bitmap)
    gdi32.DeleteDC(mem_dc)
    user32.ReleaseDC(target_hwnd, hwnd_dc)

    raw_bytes = bytearray(raw_buffer.raw)
    for i in range(0, len(raw_bytes), 4):
        b, g, r, a = raw_bytes[i:i+4]
        raw_bytes[i] = r
        raw_bytes[i+1] = g
        raw_bytes[i+2] = b
        raw_bytes[i+3] = 255

    def write_png(buf, width, height, path):
        line_len = width * 4
        raw_data = bytearray()
        for y in range(height):
            raw_data.append(0)
            raw_data.extend(buf[y * line_len:(y + 1) * line_len])
        compressed = zlib.compress(bytes(raw_data), 6)
        ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
        ihdr_crc = zlib.crc32(b'IHDR' + ihdr_data)
        idat_crc = zlib.crc32(b'IDAT' + compressed)
        iend_crc = zlib.crc32(b'IEND')
        with open(path, 'wb') as f:
            f.write(b'\x89PNG\r\n\x1a\n')
            f.write(struct.pack('>I', len(ihdr_data)) + b'IHDR' + ihdr_data + struct.pack('>I', ihdr_crc))
            f.write(struct.pack('>I', len(compressed)) + b'IDAT' + compressed + struct.pack('>I', idat_crc))
            f.write(struct.pack('>I', 0) + b'IEND' + struct.pack('>I', iend_crc))

    write_png(raw_bytes, w, h, output_path)
    print(f"Successfully captured {output_path} ({w}x{h})")
    return True

routes_to_test = [
    ("qualium://settings", r"C:\Users\mndab\.gemini\antigravity-ide\brain\38ef44a2-12b9-4657-8f35-3220c9d8b70b\qualium_v5_live_settings_route.png"),
    ("qualium://privacy", r"C:\Users\mndab\.gemini\antigravity-ide\brain\38ef44a2-12b9-4657-8f35-3220c9d8b70b\qualium_v5_live_privacy_route.png"),
    ("qualium://bookmarks", r"C:\Users\mndab\.gemini\antigravity-ide\brain\38ef44a2-12b9-4657-8f35-3220c9d8b70b\qualium_v5_live_bookmarks_route.png"),
    ("qualium://does-not-exist", r"C:\Users\mndab\.gemini\antigravity-ide\brain\38ef44a2-12b9-4657-8f35-3220c9d8b70b\qualium_v5_live_error_route.png"),
    ("https://news.ycombinator.com", r"C:\Users\mndab\.gemini\antigravity-ide\brain\38ef44a2-12b9-4657-8f35-3220c9d8b70b\qualium_v5_live_external_web.png"),
]

exe = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium\QualiumQuantumBrowser.exe")

for route, out_img in routes_to_test:
    print(f"\n--- Navigating to {route} ---")
    subprocess.run([exe, "-url", route], check=False)
    time.sleep(3.5)
    capture_screenshot(out_img)
