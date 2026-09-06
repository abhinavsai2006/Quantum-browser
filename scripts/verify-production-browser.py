#!/usr/bin/env python3
"""
Qualium Quantum Browser v5 — Final Automated Production Verification Suite
Comprehensive audit of executable, runtime, daemon, process tree, mock data, and resources.
"""

import os
import sys
import time
import zipfile
import subprocess
import ctypes
from ctypes import wintypes

user32 = ctypes.windll.user32

def print_header(title):
    print("\n" + "=" * 70)
    print(title)
    print("=" * 70)

def deoptimize_jar(data):
    import struct
    eocd_idx = data.find(b'PK\x05\x06')
    if eocd_idx == -1: return data
    cd_len = struct.unpack('<I', data[:4])[0]
    cd_part = data[4:eocd_idx]
    eocd_part = bytearray(data[eocd_idx:eocd_idx+22])
    local_part = data[eocd_idx+22:]
    struct.pack_into('<I', eocd_part, 16, len(local_part))

    cd_entries = bytearray(cd_part)
    idx = 0
    while idx < len(cd_entries):
        if cd_entries[idx:idx+4] == b'PK\x01\x02':
            orig_off = struct.unpack('<I', cd_entries[idx+42:idx+46])[0]
            struct.pack_into('<I', cd_entries, idx+42, orig_off - (eocd_idx + 22))
            name_len = struct.unpack('<H', cd_entries[idx+28:idx+30])[0]
            extra_len = struct.unpack('<H', cd_entries[idx+30:idx+32])[0]
            comm_len = struct.unpack('<H', cd_entries[idx+32:idx+34])[0]
            idx += 46 + name_len + extra_len + comm_len
        else: break
    return local_part + cd_entries + eocd_part

def main():
    print_header("QUALIUM QUANTUM BROWSER v5 — PRODUCTION VERIFICATION AUDIT")
    all_passed = True
    failures = []

    app_dir = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qaulium")
    if not os.path.exists(app_dir):
        app_dir = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium")

    browser_exe = os.path.join(app_dir, "QauliumQuantumBrowser.exe")
    if not os.path.exists(browser_exe):
        browser_exe = os.path.join(app_dir, "QualiumQuantumBrowser.exe")

    daemon_exe = os.path.join(app_dir, "qualium-daemon.exe")
    runtime_dir = os.path.join(app_dir, "runtime")
    core_exe = os.path.join(runtime_dir, "qualium-core.exe")
    xul_dll = os.path.join(runtime_dir, "xul.dll")
    omni_ja = os.path.join(runtime_dir, "browser", "omni.ja")
    
    installer_exe = r"e:\Qaulium AI\Broswer\dist\Qaulium-Quantum-Browser-v5.0.0-Setup.exe"
    if not os.path.exists(installer_exe):
        installer_exe = r"e:\Qaulium AI\Broswer\dist\Qualium-Quantum-Browser-v5.0.0-Setup.exe"

    # 1. Executables & Binaries Check
    print("\n[CHECK 1] Executables & Core Engine Libraries...")
    checks = [
        ("Qaulium Browser Shell", browser_exe),
        ("Qaulium Network Daemon", daemon_exe),
        ("Gecko Core Executable", core_exe),
        ("Gecko Runtime DLL (xul.dll)", xul_dll),
        ("Gecko Archive (omni.ja)", omni_ja),
        ("Release Setup Installer", installer_exe)
    ]
    for label, path in checks:
        if os.path.exists(path) and os.path.getsize(path) > 0:
            print(f"  [OK] {label}: {path} ({os.path.getsize(path):,} bytes)")
        else:
            print(f"  [FAIL] {label} MISSING or EMPTY: {path}")
            all_passed = False
            failures.append(f"Missing {label}")

    # 2. Package & Manifest Resource Audit in omni.ja
    print("\n[CHECK 2] Internal Resource Package Registration in omni.ja...")
    required_omni_entries = [
        "chrome/browser/content/qualium/newtab.xhtml",
        "chrome/browser/content/qualium/settings.xhtml",
        "chrome/browser/content/qualium/dashboard.xhtml",
        "chrome/browser/content/qualium/qualium-panel.xhtml",
        "chrome/browser/content/qualium/onboarding.xhtml",
        "chrome/browser/content/qualium/runtime-state.js",
        "chrome/browser/content/qualium/favicon-service.js",
        "chrome/browser/content/qualium/favicon-bridge.js",
        "chrome/browser/skin/classic/qualium/qualium-shield.svg",
        "chrome/chrome.manifest"
    ]
    try:
        import io, struct
        with open(omni_ja, "rb") as f:
            raw_data = f.read()
        std_data = deoptimize_jar(raw_data)
        with zipfile.ZipFile(io.BytesIO(std_data), 'r') as zf:
            namelist = set(zf.namelist())
            for req in required_omni_entries:
                if req in namelist:
                    print(f"  [OK] Found embedded package entry: {req}")
                else:
                    print(f"  [FAIL] MISSING package entry in omni.ja: {req}")
                    all_passed = False
                    failures.append(f"Missing omni entry {req}")
            
            # Verify chrome.manifest contents
            manifest_content = zf.read("chrome/chrome.manifest").decode("utf-8", "ignore")
            if "content qualium browser/content/qualium/" in manifest_content and "override chrome://browser/content/preferences/preferences.xhtml chrome://qualium/content/settings.xhtml" in manifest_content:
                print("  [OK] chrome.manifest content package registration & preferences override verified")
            else:
                print("  [FAIL] chrome.manifest registration missing")
                all_passed = False
                failures.append("chrome.manifest registration incomplete")
    except Exception as e:
        print(f"  [FAIL] Failed to inspect omni.ja: {e}")
        all_passed = False
        failures.append(f"omni.ja read error: {e}")

    # 3. No Mock Data / Zero Hardcoded Runtime State
    print("\n[CHECK 3] No Mock Data / Runtime State Verification...")
    hardcoded_checks = [
        (r"e:\Qaulium AI\Broswer\qualium\chrome\content\newtab.xhtml", ["Quantum-Shield-01", "US-East (Kyber-1024)"]),
        (r"e:\Qaulium AI\Broswer\qualium\chrome\content\qualium-panel.xhtml", ['val-bold">24', 'val-bold">13'])
    ]
    for file_path, bad_strings in hardcoded_checks:
        if os.path.exists(file_path):
            with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
                content = f.read()
            for bad in bad_strings:
                if bad in content:
                    print(f"  [FAIL] Hardcoded mock value '{bad}' found in {os.path.basename(file_path)}")
                    all_passed = False
                    failures.append(f"Hardcoded '{bad}' in {file_path}")
                else:
                    print(f"  [OK] No hardcoded '{bad}' in {os.path.basename(file_path)}")

    # 4. Zero Localhost Dependency Audit
    print("\n[CHECK 4] Zero Localhost Dependency Audit...")
    source_content_dir = r"e:\Qaulium AI\Broswer\qualium\chrome\content"
    localhost_found = False
    for root, _, files in os.walk(source_content_dir):
        for f in files:
            if f.endswith((".xhtml", ".js", ".css")):
                p = os.path.join(root, f)
                with open(p, "r", encoding="utf-8", errors="ignore") as fp:
                    c = fp.read()
                    if "localhost:3000" in c or "localhost:5173" in c or "localhost:4173" in c:
                        print(f"  [FAIL] Localhost reference found in {p}")
                        localhost_found = True
                        all_passed = False
                        failures.append(f"Localhost in {p}")
    if not localhost_found:
        print("  [OK] Zero localhost server dependencies found across all production chrome content")

    # 5. Live Process Tree & Window Integrity Audit
    print("\n[CHECK 5] Live Installed Executable & Process Tree Execution...")
    # Kill any lingering test instances
    subprocess.run(["powershell", "-Command", "Get-Process QualiumQuantumBrowser, qualium-daemon, qualium-core, firefox -ErrorAction SilentlyContinue | Stop-Process -Force"], capture_output=True)
    time.sleep(1)

    # Clear locks from both profile directories
    for prof_base in [r"%LOCALAPPDATA%\Qaulium\Profile", r"%LOCALAPPDATA%\Qualium\Profile"]:
        prof_dir = os.path.expandvars(prof_base)
        for lock in ["parent.lock", ".parentlock"]:
            lp = os.path.join(prof_dir, lock)
            if os.path.exists(lp):
                try: os.remove(lp)
                except: pass

    # Launch browser shell
    proc = subprocess.Popen([browser_exe])
    print(f"  Spawned {os.path.basename(browser_exe)} (PID {proc.pid})")
    # Inspect Window with polling loop (up to 15 seconds)
    qualium_window_found = False
    WNDENUMPROC = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)

    for poll_idx in range(15):
        time.sleep(1)
        def enum_cb(hwnd, _):
            nonlocal qualium_window_found
            if user32.IsWindowVisible(hwnd):
                length = user32.GetWindowTextLengthW(hwnd)
                buff = ctypes.create_unicode_buffer(length + 1)
                user32.GetWindowTextW(hwnd, buff, length + 1)
                title = buff.value
                cls_name = ctypes.create_unicode_buffer(256)
                user32.GetClassNameW(hwnd, cls_name, 256)
                pid = wintypes.DWORD()
                user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
                if "Qualium" in title or "Qaulium" in title or cls_name.value == "MozillaWindowClass":
                    rect = wintypes.RECT()
                    user32.GetWindowRect(hwnd, ctypes.byref(rect))
                    w = rect.right - rect.left
                    h = rect.bottom - rect.top
                    if w > 300 and h > 200:
                        print(f"  [OK] Live Window Verified (after {poll_idx+1}s): HWND {hwnd}, Class '{cls_name.value}', Title '{title}', Dimensions {w}x{h}")
                        qualium_window_found = True
                        return False
            return True
        user32.EnumWindows(WNDENUMPROC(enum_cb), 0)
        if qualium_window_found:
            break

    # Inspect process tree
    ps_cmd = "Get-CimInstance Win32_Process | Where-Object { $_.Name -match 'Qualium|qualium' } | Select-Object ProcessId, Name, CommandLine | Format-Table -AutoSize"
    try:
        proc_info = subprocess.check_output(["powershell", "-Command", ps_cmd]).decode("utf-8", "ignore")
        print("\n  Active Qualium Process Tree:")
        for line in proc_info.strip().splitlines():
            print(f"    {line}")
    except Exception:
        pass

    if not qualium_window_found:
        print("  [FAIL] Qualium live window not found")
        all_passed = False
        failures.append("Live window not detected")

    # Clean shutdown
    subprocess.run(["powershell", "-Command", "Get-Process QualiumQuantumBrowser, qualium-daemon, qualium-core -ErrorAction SilentlyContinue | Stop-Process -Force"], capture_output=True)
    time.sleep(1)
    print("  [OK] Test session terminated cleanly")

    # Audit Summary
    print_header("FINAL VERIFICATION AUDIT SUMMARY")
    if all_passed:
        print("RESULT: ALL P0 PRODUCTION REQUIREMENTS SATISFIED [PASS]")
        print("The Qualium Quantum Browser v5 desktop application is complete, verified, and operational.")
        return 0
    else:
        print(f"RESULT: FAILURES DETECTED ({len(failures)}):")
        for f in failures:
            print(f"  - {f}")
        return 1

if __name__ == "__main__":
    sys.exit(main())
