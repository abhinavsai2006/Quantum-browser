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
    print("  QUALIUM QUANTUM BROWSER — URL SYNCHRONIZATION & TAB TEST PIPELINE    ")
    print("========================================================================")
    kill_qualium()

    exe = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium\QualiumQuantumBrowser.exe")
    if not os.path.exists(exe):
        exe = r"e:\Qaulium AI\Broswer\QualiumQuantumBrowser.exe"

    print(f"\n[Phase 1] Launching clean Qualium browser instance from:\n  {exe}")
    proc = subprocess.Popen([exe])
    time.sleep(5.0)

    hwnd = find_qualium_window(15.0)
    if not hwnd:
        print("[-] ERROR: Qualium window could not be located!")
        kill_qualium()
        return False

    user32.SetWindowPos(hwnd, 0, 40, 40, 1280, 840, 0x0040)
    time.sleep(1.0)

    # 1. Verify Initial Empty New Tab
    print("\n[Phase 2] Verifying initial empty New Tab...")
    info1 = poll_page_info(10.0)
    print(f"  TabCount : {info1.get('tabCount')}")
    print(f"  URI      : {info1.get('uri')}")
    print(f"  Title    : {info1.get('title')}")
    print(f"  Urlbar   : {info1.get('urlbar')}")
    step1_png = os.path.join(ARTIFACT_DIR, "v2_step1_initial_tab.png")
    capture_rect_screenshot(hwnd, step1_png)

    assert info1.get("tabCount") == 1, f"Expected 1 tab, got {info1.get('tabCount')}"
    assert info1.get("urlbar") == "qualium://newtab", f"Expected qualium://newtab in urlbar, got: {info1.get('urlbar')}"
    assert "New Tab" in info1.get("title", ""), f"Expected 'New Tab' title, got: {info1.get('title')}"
    print("  [PASS] Initial New Tab verified: URL bar is qualium://newtab and Title is 'New Tab'")

    # 2. Search Google for "you" from New Tab
    print("\n[Phase 3] Searching 'you' in the active tab...")
    send_bridge_cmd("SEARCH_FROM_PAGE:you")
    time.sleep(5.5)

    info2 = poll_page_info(12.0)
    print(f"  TabCount : {info2.get('tabCount')}")
    print(f"  URI      : {info2.get('uri')}")
    print(f"  Title    : {info2.get('title')}")
    print(f"  Urlbar   : {info2.get('urlbar')}")
    step2_png = os.path.join(ARTIFACT_DIR, "v2_step2_google_search.png")
    capture_rect_screenshot(hwnd, step2_png)

    assert info2.get("tabCount") == 1, f"Search must stay in single tab! Got: {info2.get('tabCount')}"
    assert "google.com" in info2.get("uri", "").lower(), f"Expected Google search URI, got: {info2.get('uri')}"
    assert info2.get("urlbar") != "qualium://newtab", f"BUG: Urlbar still shows qualium://newtab after searching!"
    assert "google.com" in info2.get("urlbar", "").lower() or "you" in info2.get("urlbar", "").lower(), f"Urlbar does not show real Google search URL: {info2.get('urlbar')}"
    # Verify title is clean and free from duplication
    title = info2.get("title", "")
    assert not title.endswith("New Tab") and not title.endswith("New T"), f"BUG: Title duplication detected: '{title}'"
    print("  [PASS] Google search verified: URL bar shows real Google URL, title has no duplication, and tab count is 1.")

    # 3. Open link in a new tab (target="_blank" simulation)
    print("\n[Phase 4] Opening a link in a new tab (https://en.wikipedia.org/wiki/Quantum_computing)...")
    wiki_url = "https://en.wikipedia.org/wiki/Quantum_computing"
    send_bridge_cmd(f"OPEN_IN_NEW_TAB:{wiki_url}")
    time.sleep(5.0)

    info3 = poll_page_info(10.0)
    print(f"  TabCount : {info3.get('tabCount')}")
    print(f"  URI      : {info3.get('uri')}")
    print(f"  Title    : {info3.get('title')}")
    print(f"  Urlbar   : {info3.get('urlbar')}")
    step3_png = os.path.join(ARTIFACT_DIR, "v2_step3_opened_link_in_new_tab.png")
    capture_rect_screenshot(hwnd, step3_png)

    assert info3.get("tabCount") == 2, f"Expected 2 tabs, got: {info3.get('tabCount')}"
    assert "wikipedia.org" in info3.get("uri", "").lower(), f"New tab URI is not Wikipedia: {info3.get('uri')}"
    assert info3.get("urlbar") != "qualium://newtab", f"BUG: New tab urlbar stuck on qualium://newtab!"
    assert "wikipedia.org" in info3.get("urlbar", "").lower(), f"New tab urlbar should show wikipedia URL, got: {info3.get('urlbar')}"
    print("  [PASS] Link in new tab verified: loaded real destination URL and URL bar reflects destination.")

    # 4. Switch back to Google tab and verify URL bar sync
    print("\n[Phase 5] Switching back to Tab 0 (Google Search)...")
    send_bridge_cmd("SWITCH_TAB:0")
    time.sleep(2.0)

    info4 = poll_page_info(8.0)
    print(f"  TabCount : {info4.get('tabCount')}")
    print(f"  URI      : {info4.get('uri')}")
    print(f"  Title    : {info4.get('title')}")
    print(f"  Urlbar   : {info4.get('urlbar')}")
    step4_png = os.path.join(ARTIFACT_DIR, "v2_step4_switch_to_google_tab.png")
    capture_rect_screenshot(hwnd, step4_png)

    assert "google.com" in info4.get("uri", "").lower(), f"Tab 0 URI should be Google, got: {info4.get('uri')}"
    assert "google.com" in info4.get("urlbar", "").lower(), f"Tab 0 URL bar should show Google search URL, got: {info4.get('urlbar')}"
    print("  [PASS] Tab switch verified: URL bar instantly updated to Tab 0 Google URL.")

    # 5. Switch to Tab 1 (Wikipedia) and verify URL bar sync
    print("\n[Phase 6] Switching to Tab 1 (Wikipedia)...")
    send_bridge_cmd("SWITCH_TAB:1")
    time.sleep(2.0)

    info5 = poll_page_info(8.0)
    print(f"  TabCount : {info5.get('tabCount')}")
    print(f"  URI      : {info5.get('uri')}")
    print(f"  Title    : {info5.get('title')}")
    print(f"  Urlbar   : {info5.get('urlbar')}")
    step5_png = os.path.join(ARTIFACT_DIR, "v2_step5_switch_to_wiki_tab.png")
    capture_rect_screenshot(hwnd, step5_png)

    assert "wikipedia.org" in info5.get("uri", "").lower(), f"Tab 1 URI should be Wikipedia, got: {info5.get('uri')}"
    assert "wikipedia.org" in info5.get("urlbar", "").lower(), f"Tab 1 URL bar should show Wikipedia URL, got: {info5.get('urlbar')}"
    print("  [PASS] Tab switch verified: URL bar instantly updated to Tab 1 Wikipedia URL.")

    # 6. Open a fresh empty tab and verify qualium://newtab
    print("\n[Phase 7] Opening a fresh empty tab via NEW_TAB...")
    send_bridge_cmd("NEW_TAB")
    time.sleep(2.5)

    info6 = poll_page_info(8.0)
    print(f"  TabCount : {info6.get('tabCount')}")
    print(f"  URI      : {info6.get('uri')}")
    print(f"  Title    : {info6.get('title')}")
    print(f"  Urlbar   : {info6.get('urlbar')}")
    step6_png = os.path.join(ARTIFACT_DIR, "v2_step6_fresh_newtab.png")
    capture_rect_screenshot(hwnd, step6_png)

    assert info6.get("tabCount") == 3, f"Expected 3 tabs, got: {info6.get('tabCount')}"
    assert info6.get("urlbar") == "qualium://newtab", f"Empty new tab must have qualium://newtab in URL bar, got: {info6.get('urlbar')}"
    assert "New Tab" in info6.get("title", ""), f"Empty new tab must have 'New Tab' title, got: {info6.get('title')}"
    print("  [PASS] Fresh empty tab verified: URL bar is qualium://newtab and title is 'New Tab'.")

    kill_qualium()

    print("\n========================================================================")
    print("  [SUCCESS] ALL URL SYNCHRONIZATION AND NAVIGATION TESTS PASSED!       ")
    print("========================================================================")
    return True

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
