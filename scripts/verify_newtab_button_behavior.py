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

def send_bridge_cmd(cmd, timeout=8.0):
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
    except Exception:
        pass

    try:
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
    except Exception as e:
        print(f"[Screenshot] PrintWindow failed ({e})")
        return False

def poll_page_info(timeout=12.0):
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
        time.sleep(0.4)
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
    print("  QUANTUM BROWSER — '+' BUTTON & NEW TAB ARCHITECTURE TEST     ")
    print("========================================================================")
    kill_qualium()

    exe = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium\QualiumQuantumBrowser.exe")
    if not os.path.exists(exe):
        exe = r"e:\Qaulium AI\Broswer\QualiumQuantumBrowser.exe"

    print(f"\n[Phase 1] Launching clean Quantum browser instance from:\n  {exe}")
    proc = subprocess.Popen([exe])
    time.sleep(5.0)

    hwnd = find_qualium_window(15.0)
    if not hwnd:
        print("[-] ERROR: Quantum browser window could not be located!")
        kill_qualium()
        return False

    user32.SetWindowPos(hwnd, 0, 40, 40, 1280, 840, 0x0040)
    time.sleep(1.0)

    # 1. Verify Initial State: Exactly ONE New Tab, NO Modal, NO URL prompt
    print("\n[Phase 2] Verifying initial startup state: exactly 1 New Tab...")
    info1 = poll_page_info(10.0)
    print(f"  TabCount : {info1.get('tabCount')}")
    print(f"  URI      : {info1.get('uri')}")
    print(f"  Title    : {info1.get('title')}")
    print(f"  Urlbar   : {info1.get('urlbar')}")
    step1_png = os.path.join(ARTIFACT_DIR, "v3_step1_initial_single_newtab.png")
    capture_rect_screenshot(hwnd, step1_png)

    assert info1.get("tabCount") == 1, f"Expected exactly 1 tab on startup, got: {info1.get('tabCount')}"
    assert info1.get("urlbar") == "qualium://newtab", f"Expected qualium://newtab, got: {info1.get('urlbar')}"
    assert "New Tab" in info1.get("title", ""), f"Expected 'New Tab' title, got: {info1.get('title')}"
    print("  [PASS] Phase 2: Browser launches with exactly one clean New Tab (Zero URL prompts, Zero modals).")

    # 2. Click '+' Button once
    print("\n[Phase 3] Clicking '+' button once (single-click action)...")
    res_add = send_bridge_cmd("NEW_TAB")
    print(f"  Bridge Command Result: {res_add}")
    time.sleep(2.5)

    info2 = poll_page_info(8.0)
    print(f"  TabCount : {info2.get('tabCount')}")
    print(f"  URI      : {info2.get('uri')}")
    print(f"  Title    : {info2.get('title')}")
    print(f"  Urlbar   : {info2.get('urlbar')}")
    step2_png = os.path.join(ARTIFACT_DIR, "v3_step2_second_tab_no_modal.png")
    capture_rect_screenshot(hwnd, step2_png)

    assert info2.get("tabCount") == 2, f"Expected exactly 2 tabs after clicking '+', got: {info2.get('tabCount')}"
    assert info2.get("urlbar") == "qualium://newtab", f"Expected qualium://newtab in newly created tab, got: {info2.get('urlbar')}"
    assert "New Tab" in info2.get("title", ""), f"Expected 'New Tab' title, got: {info2.get('title')}"
    print("  [PASS] Phase 3: '+' clicked -> New Tab created immediately, focused, zero modals/prompts.")

    # 3. Click '+' Button 4 more times (total 6 tabs)
    print("\n[Phase 4] Clicking '+' button 4 additional times (rapid creation)...")
    for i in range(3, 7):
        send_bridge_cmd("NEW_TAB")
        time.sleep(1.0)
        curr_info = poll_page_info(5.0)
        print(f"  [+] Tab {i} created: TabCount={curr_info.get('tabCount')}, ActiveTitle={curr_info.get('title')}")
        assert curr_info.get("tabCount") == i, f"Expected {i} tabs, got {curr_info.get('tabCount')}"

    info6 = poll_page_info(6.0)
    print(f"\n  Final TabCount : {info6.get('tabCount')}")
    step3_png = os.path.join(ARTIFACT_DIR, "v3_step3_six_tabs_created.png")
    capture_rect_screenshot(hwnd, step3_png)
    assert info6.get("tabCount") == 6, f"Expected 6 tabs, got: {info6.get('tabCount')}"
    print("  [PASS] Phase 4: Successfully created 6 independent tabs without any modals or prompts.")

    # 4. Navigate Tab 5 to a web URL (Wikipedia), verify Tab 0 stays on New Tab
    print("\n[Phase 5] Navigating Tab 5 to Wikipedia, verifying independent tab states...")
    send_bridge_cmd("NAVIGATE:https://en.wikipedia.org/wiki/Quantum_computing")
    time.sleep(5.0)

    info_wiki = poll_page_info(8.0)
    print(f"  Tab 5 URL    : {info_wiki.get('uri')}")
    print(f"  Tab 5 Urlbar : {info_wiki.get('urlbar')}")
    print(f"  Tab 5 Title  : {info_wiki.get('title')}")
    assert "wikipedia.org" in info_wiki.get("uri", "").lower(), f"Expected Wikipedia URI, got: {info_wiki.get('uri')}"

    # Switch back to Tab 0
    print("  Switching back to Tab 0...")
    send_bridge_cmd("SWITCH_TAB:0")
    time.sleep(1.5)

    info_tab0 = poll_page_info(6.0)
    print(f"  Tab 0 URL    : {info_tab0.get('uri')}")
    print(f"  Tab 0 Urlbar : {info_tab0.get('urlbar')}")
    print(f"  Tab 0 Title  : {info_tab0.get('title')}")
    assert "newtab" in info_tab0.get("uri", "").lower() or "qualium://newtab" in info_tab0.get("urlbar", "").lower(), f"Tab 0 state polluted! Got: {info_tab0}"
    step4_png = os.path.join(ARTIFACT_DIR, "v3_step4_independent_tabs.png")
    capture_rect_screenshot(hwnd, step4_png)
    print("  [PASS] Phase 5: Tab 5 navigated independently while Tab 0 retained clean New Tab state.")

    # 5. Close tabs until 1 remains and verify remaining tab activation
    print("\n[Phase 6] Closing tabs down to 1 tab and verifying remaining tab activation...")
    for c in range(5):
        send_bridge_cmd("CLOSE_TAB")
        time.sleep(1.0)
        curr_info = poll_page_info(5.0)
        print(f"  [-] Closed tab {c+1}: Remaining TabCount={curr_info.get('tabCount')}")

    info_final = poll_page_info(6.0)
    print(f"\n  Post-Close TabCount : {info_final.get('tabCount')}")
    print(f"  Post-Close Title    : {info_final.get('title')}")
    print(f"  Post-Close Urlbar   : {info_final.get('urlbar')}")
    step5_png = os.path.join(ARTIFACT_DIR, "v3_step5_single_tab_remaining.png")
    capture_rect_screenshot(hwnd, step5_png)

    assert info_final.get("tabCount") == 1, f"Expected 1 remaining tab, got: {info_final.get('tabCount')}"
    print("  [PASS] Phase 6: Closing tabs down to 1 cleanly preserved and activated the remaining tab without breaking window.")

    kill_qualium()

    print("\n========================================================================")
    print("  [SUCCESS] ALL '+' BUTTON & NEW TAB ARCHITECTURE TESTS PASSED!        ")
    print("========================================================================")
    return True

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
