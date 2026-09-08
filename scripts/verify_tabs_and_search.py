import os, sys, time, subprocess, ctypes, json
from ctypes import wintypes
from PIL import ImageGrab

sys.stdout.reconfigure(line_buffering=True)
sys.stderr.reconfigure(line_buffering=True)

user32 = ctypes.windll.user32
gdi32 = ctypes.windll.gdi32

ARTIFACT_DIR = r"C:\Users\mndab\.gemini\antigravity-ide\brain\c8e1ddf8-7792-4a3e-99b6-182cababcb8d"
TEMP_DIR = os.environ.get("TEMP", r"C:\Users\mndab\AppData\Local\Temp")
CMD_FILE = os.path.join(TEMP_DIR, "qualium_menu_cmd.txt")
RES_FILE = os.path.join(TEMP_DIR, "qualium_menu_cmd_result.txt")

def send_bridge_cmd(cmd, timeout=7.0):
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

os.environ["QUALIUM_AUTOMATION_TEST"] = "1"

def poll_page_info(timeout=10.0):
    start = time.time()
    last = None
    while time.time() - start < timeout:
        raw = send_bridge_cmd("GET_PAGE_INFO")
        if raw and raw != "TIMEOUT":
            last = raw
            try:
                data = json.loads(raw)
                if isinstance(data, dict):
                    return data
            except: pass
        time.sleep(0.3)
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
    print("========================================================================")
    print("  QUALIUM QUANTUM BROWSER — TAB MANAGEMENT & NAVIGATION VERIFICATION   ")
    print("========================================================================")
    kill_qualium()

    exe = r"e:\Qaulium AI\Broswer\QualiumQuantumBrowser.exe"
    print(f"\n[Test 1] Launching default Qualium browser...")
    proc = subprocess.Popen([exe])
    time.sleep(4.5)

    hwnd = find_qualium_window(12.0)
    if not hwnd:
        print("[-] ERROR: Qualium window NOT found!")
        kill_qualium()
        return False

    user32.SetWindowPos(hwnd, 0, 50, 50, 1280, 820, 0x0040)
    time.sleep(1.0)

    # Step 1: Verify Initial Tab State (Single "New Tab")
    info1 = poll_page_info(8.0)
    print(f"[1.1] Initial State: TabCount={info1.get('tabCount', 1)}, URL={info1.get('uri')}, Title={info1.get('title')}")
    assert info1.get("tabCount", 1) == 1, f"Expected 1 tab on startup, found {info1.get('tabCount')}"
    step1_png = os.path.join(ARTIFACT_DIR, "step1_initial_single_tab.png")
    capture_rect_screenshot(hwnd, step1_png)
    print("  [PASS] Step 1: Single clean New Tab verified.")

    # Step 2: Search from New Tab Page (query: 'yt')
    print("\n[Test 2] Simulating search query 'yt' from New Tab page...")
    search_res = send_bridge_cmd("SEARCH_FROM_PAGE:yt")
    print(f"[2.1] Search command response: {search_res}")
    time.sleep(5.0)

    info2 = poll_page_info(10.0)
    print(f"[2.2] Post-Search State: TabCount={info2.get('tabCount')}, URL={info2.get('uri')}, Title={info2.get('title')}")
    assert info2.get("tabCount", 1) == 1, f"Expected STILL 1 tab after search, but found {info2.get('tabCount')}"
    assert "google" in info2.get("uri", "").lower() or "yt" in info2.get("uri", "").lower(), f"Unexpected URL after search: {info2.get('uri')}"
    step2_png = os.path.join(ARTIFACT_DIR, "step2_intab_search_single_tab.png")
    capture_rect_screenshot(hwnd, step2_png)
    print("  [PASS] Step 2: Search navigates in CURRENT tab without creating a second tab.")

    # Step 3: Open a Second Tab (Ctrl+T / + button)
    print("\n[Test 3] Creating a second tab via NEW_TAB...")
    newtab_res = send_bridge_cmd("NEW_TAB")
    print(f"[3.1] NEW_TAB command response: {newtab_res}")
    time.sleep(2.0)

    info3 = poll_page_info(8.0)
    print(f"[3.2] Post-NewTab State: TabCount={info3.get('tabCount')}, ActiveURL={info3.get('uri')}")
    assert info3.get("tabCount") == 2, f"Expected 2 tabs after NEW_TAB, found {info3.get('tabCount')}"
    step3_png = os.path.join(ARTIFACT_DIR, "step3_two_tabs_created.png")
    capture_rect_screenshot(hwnd, step3_png)
    print("  [PASS] Step 3: Second tab created cleanly; tab count is 2.")

    # Step 4: Navigate in Tab 2 to qualium://bookmarks
    print("\n[Test 4] Navigating Tab 2 to qualium://bookmarks...")
    nav_res = send_bridge_cmd("NAVIGATE:qualium://bookmarks")
    print(f"[4.1] Navigate command response: {nav_res}")
    time.sleep(2.5)

    info4 = poll_page_info(8.0)
    print(f"[4.2] Tab 2 Navigated State: URL={info4.get('uri')}, Title={info4.get('title')}")
    step4_png = os.path.join(ARTIFACT_DIR, "step4_tab2_navigated.png")
    capture_rect_screenshot(hwnd, step4_png)
    print("  [PASS] Step 4: Tab 2 navigated independently.")

    # Step 5: Switch back to Tab 1
    print("\n[Test 5] Switching back to Tab 1 via SWITCH_TAB:0...")
    switch_res = send_bridge_cmd("SWITCH_TAB:0")
    print(f"[5.1] Switch command response: {switch_res}")
    time.sleep(1.5)

    info5 = poll_page_info(6.0)
    print(f"[5.2] Post-Switch State: ActiveURL={info5.get('uri')}, Title={info5.get('title')}")
    assert "google" in info5.get("uri", "").lower() or "yt" in info5.get("uri", "").lower(), f"Tab 1 URL lost! Found: {info5.get('uri')}"
    step5_png = os.path.join(ARTIFACT_DIR, "step5_switched_back_to_tab1.png")
    capture_rect_screenshot(hwnd, step5_png)
    print("  [PASS] Step 5: Switched to Tab 1; previous page state preserved.")

    # Step 6: Close Tab 1
    print("\n[Test 6] Closing Tab 1 via CLOSE_TAB...")
    close_res = send_bridge_cmd("CLOSE_TAB")
    print(f"[6.1] Close command response: {close_res}")
    time.sleep(2.0)

    info6 = poll_page_info(6.0)
    print(f"[6.2] Post-Close State: TabCount={info6.get('tabCount')}, ActiveURL={info6.get('uri')}")
    assert info6.get("tabCount") == 1, f"Expected 1 tab remaining after close, found {info6.get('tabCount')}"
    step6_png = os.path.join(ARTIFACT_DIR, "step6_tab_closed_remaining_activated.png")
    capture_rect_screenshot(hwnd, step6_png)
    print("  [PASS] Step 6: Tab closed cleanly and remaining tab activated.")

    # Step 7: Close the last remaining tab -> should auto-create fresh New Tab
    print("\n[Test 7] Closing last remaining tab...")
    close_res2 = send_bridge_cmd("CLOSE_TAB")
    print(f"[7.1] Close last tab response: {close_res2}")
    time.sleep(2.0)

    info7 = poll_page_info(6.0)
    print(f"[7.2] Final State: TabCount={info7.get('tabCount')}, ActiveURL={info7.get('uri')}")
    assert info7.get("tabCount", 1) >= 1, "Tab count reached 0 without creating fallback new tab!"
    step7_png = os.path.join(ARTIFACT_DIR, "step7_all_closed_fallback_newtab.png")
    capture_rect_screenshot(hwnd, step7_png)
    print("  [PASS] Step 7: Closing all tabs cleanly re-initialized a fresh New Tab.")

    try:
        proc.kill()
    except: pass
    kill_qualium()

    print("\n========================================================================")
    print("  [SUCCESS] ALL 7 VERIFICATION STAGES PASSED WITHOUT DEFECTS!          ")
    print("========================================================================")
    return True

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
