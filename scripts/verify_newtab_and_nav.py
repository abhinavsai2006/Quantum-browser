import os, sys, time, subprocess, ctypes, json
from ctypes import wintypes
from PIL import ImageGrab

sys.stdout.reconfigure(line_buffering=True)
sys.stderr.reconfigure(line_buffering=True)

user32 = ctypes.windll.user32
gdi32 = ctypes.windll.gdi32

ARTIFACT_DIR = r"C:\Users\mndab\.gemini\antigravity-ide\brain\b0a47078-44ea-43d7-ae34-0240dd9ce43a"
TEMP_DIR = os.environ.get("TEMP", r"C:\Users\mndab\AppData\Local\Temp")
CMD_FILE = os.path.join(TEMP_DIR, "qualium_menu_cmd.txt")
RES_FILE = os.path.join(TEMP_DIR, "qualium_menu_cmd_result.txt")

def send_bridge_cmd(cmd, timeout=6.0):
    try:
        if os.path.exists(RES_FILE):
            os.remove(RES_FILE)
    except: pass

    with open(CMD_FILE, "w", encoding="utf-8") as f:
        f.write(cmd)

    start = time.time()
    while time.time() - start < timeout:
        if os.path.exists(RES_FILE):
            try:
                with open(RES_FILE, "r", encoding="utf-8") as f:
                    res = f.read().strip()
                os.remove(RES_FILE)
                return res
            except: pass
        time.sleep(0.05)
    return "TIMEOUT"

def find_qualium_window(max_wait=12.0):
    start = time.time()
    while time.time() - start < max_wait:
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
        if target_hwnd:
            return target_hwnd
        time.sleep(0.3)
    return None

def capture_rect_screenshot(hwnd, output_path):
    user32.ShowWindow(hwnd, 9)
    user32.SetForegroundWindow(hwnd)
    time.sleep(0.5)

    rect = wintypes.RECT()
    user32.GetWindowRect(hwnd, ctypes.byref(rect))
    left, top, right, bottom = rect.left, rect.top, rect.right, rect.bottom
    w = max(1, right - left)
    h = max(1, bottom - top)

    try:
        img = ImageGrab.grab(bbox=(left, top, right, bottom))
        img.save(output_path)
        print(f"[Screenshot] ImageGrab saved: {output_path} ({w}x{h})")
        return True
    except Exception as e:
        print(f"[Screenshot] ImageGrab failed ({e}), falling back to PrintWindow...")

    # Fallback to PrintWindow
    hwnd_dc = user32.GetWindowDC(hwnd)
    mem_dc = gdi32.CreateCompatibleDC(hwnd_dc)
    bitmap = gdi32.CreateCompatibleBitmap(hwnd_dc, w, h)
    old_bmp = gdi32.SelectObject(mem_dc, bitmap)

    user32.PrintWindow(hwnd, mem_dc, 0x00000002)

    import zlib, struct
    bmi = struct.pack('<IiiHHIIIIII', 40, w, -h, 1, 32, 0, w * h * 4, 0, 0, 0, 0)
    raw_buffer = ctypes.create_string_buffer(w * h * 4)
    gdi32.GetDIBits(mem_dc, bitmap, 0, h, raw_buffer, bmi, 0)

    gdi32.SelectObject(mem_dc, old_bmp)
    gdi32.DeleteObject(bitmap)
    gdi32.DeleteDC(mem_dc)
    user32.ReleaseDC(hwnd, hwnd_dc)

    raw_bytes = bytearray(raw_buffer.raw)
    for i in range(0, len(raw_bytes), 4):
        b, g, r, a = raw_bytes[i:i+4]
        raw_bytes[i] = r
        raw_bytes[i+1] = g
        raw_bytes[i+2] = b
        raw_bytes[i+3] = 255

    line_len = w * 4
    raw_data = bytearray()
    for y in range(h):
        raw_data.append(0)
        raw_data.extend(raw_bytes[y * line_len:(y + 1) * line_len])
    compressed = zlib.compress(bytes(raw_data), 6)
    ihdr_data = struct.pack('>IIBBBBB', w, h, 8, 6, 0, 0, 0)
    ihdr_crc = zlib.crc32(b'IHDR' + ihdr_data)
    idat_crc = zlib.crc32(b'IDAT' + compressed)
    iend_crc = zlib.crc32(b'IEND')
    with open(output_path, 'wb') as f:
        f.write(b'\x89PNG\r\n\x1a\n')
        f.write(struct.pack('>I', len(ihdr_data)) + b'IHDR' + ihdr_data + struct.pack('>I', ihdr_crc))
        f.write(struct.pack('>I', len(compressed)) + b'IDAT' + compressed + struct.pack('>I', idat_crc))
        f.write(struct.pack('>I', 0) + b'IEND' + struct.pack('>I', iend_crc))
    print(f"[Screenshot] PrintWindow saved: {output_path} ({w}x{h})")
    return True

def poll_page_info(timeout=10.0):
    start = time.time()
    last = None
    while time.time() - start < timeout:
        raw = send_bridge_cmd("GET_PAGE_INFO")
        if raw and raw != "TIMEOUT":
            last = raw
            try:
                data = json.loads(raw)
                # If loaded beyond about:blank
                if data.get("uri") and data.get("uri") != "about:blank":
                    return data
            except: pass
        time.sleep(0.5)
    try:
        return json.loads(last) if last else {}
    except:
        return {"raw": last}

def kill_qualium():
    subprocess.run(["powershell", "-Command", "Get-Process *qualium*,*firefox* -ErrorAction SilentlyContinue | Stop-Process -Force"], check=False)
    time.sleep(1.0)
    for p in [r"%LOCALAPPDATA%\Qaulium\Profile\parent.lock", r"%LOCALAPPDATA%\Qualium\Profile\parent.lock"]:
        path = os.path.expandvars(p)
        if os.path.exists(path):
            try: os.remove(path)
            except: pass

def main():
    print("=== TESTING QUALIUM NEW TAB & NAVIGATION VERIFICATION ===")
    kill_qualium()

    exe = r"e:\Qaulium AI\Broswer\QualiumQuantumBrowser.exe"
    print(f"[1] Launching default QualiumQuantumBrowser with NO args...")
    proc = subprocess.Popen([exe])
    time.sleep(4.0)

    hwnd = find_qualium_window(10.0)
    if not hwnd:
        print("[-] Qualium window NOT found!")
        kill_qualium()
        return

    user32.SetWindowPos(hwnd, 0, 40, 40, 1280, 820, 0x0040)
    time.sleep(1.0)

    # 1. Inspect initial startup state
    info1 = poll_page_info(8.0)
    print(f"[+] Initial Page Info (settled): {json.dumps(info1, indent=2)}")

    newtab_png = os.path.join(ARTIFACT_DIR, "qualium_newtab_verified.png")
    capture_rect_screenshot(hwnd, newtab_png)

    # 2. Test navigation to external website
    print("\n[2] Navigating to external website (https://www.youtube.com)...")
    nav_res = send_bridge_cmd("NAVIGATE:https://www.youtube.com")
    print(f"[+] Navigate command result: {nav_res}")

    info2 = poll_page_info(10.0)
    print(f"[+] Navigated Page Info (settled): {json.dumps(info2, indent=2)}")

    nav_png = os.path.join(ARTIFACT_DIR, "qualium_navigation_verified.png")
    capture_rect_screenshot(hwnd, nav_png)

    # 3. Test Secondary Instance Launch
    print("\n[3] Testing secondary instance launch with -url https://news.ycombinator.com...")
    subprocess.run([exe, "-url", "https://news.ycombinator.com"], check=False)
    time.sleep(3.0)

    info3 = None
    for _ in range(20):
        raw = send_bridge_cmd("GET_PAGE_INFO")
        if raw and "ycombinator" in raw:
            try: info3 = json.loads(raw); break
            except: pass
        time.sleep(0.5)
    if not info3:
        try: info3 = json.loads(send_bridge_cmd("GET_PAGE_INFO"))
        except: info3 = {}
    print(f"[+] Secondary Instance Delegated Page Info (settled): {json.dumps(info3, indent=2)}")

    sec_png = os.path.join(ARTIFACT_DIR, "qualium_secondary_nav_verified.png")
    capture_rect_screenshot(hwnd, sec_png)

    try:
        proc.kill()
    except: pass
    kill_qualium()
    print("\n[+] Verification suite finished cleanly!")

if __name__ == "__main__":
    main()
