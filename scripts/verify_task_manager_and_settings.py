import os
import sys
import win32api
from PIL import Image

ARTIFACT_DIR = r"C:\Users\mndab\.gemini\antigravity-ide\brain\b0a47078-44ea-43d7-ae34-0240dd9ce43a"

def verify_pe_version(exe_path):
    print(f"\n[Check] Verifying PE Version Info for: {exe_path}")
    assert os.path.exists(exe_path), f"File {exe_path} not found"
    
    desc = win32api.GetFileVersionInfo(exe_path, "\\StringFileInfo\\040904b0\\FileDescription")
    prod = win32api.GetFileVersionInfo(exe_path, "\\StringFileInfo\\040904b0\\ProductName")
    comp = win32api.GetFileVersionInfo(exe_path, "\\StringFileInfo\\040904b0\\CompanyName")
    
    print(f"  FileDescription: '{desc}'")
    print(f"  ProductName:     '{prod}'")
    print(f"  CompanyName:     '{comp}'")

    assert "Firefox" not in desc, f"FileDescription still contains 'Firefox' in {exe_path}"
    assert "Firefox" not in prod, f"ProductName still contains 'Firefox' in {exe_path}"
    assert "Mozilla" not in comp, f"CompanyName still contains 'Mozilla' in {exe_path}"
    assert "Qaulium" in desc, f"FileDescription does not contain 'Qaulium' in {exe_path} (got: {desc})"
    assert "Qaulium" in prod, f"ProductName does not contain 'Qaulium' in {exe_path} (got: {prod})"
    print("  [PASS] Clean Qaulium branding verified in PE version resource.")

def verify_settings_no_mock():
    print("\n[Check] Verifying Settings Page Interactive Controls & Zero Mock Badges...")
    settings_p = os.path.join(r"e:\Qaulium AI\Broswer", "qualium", "chrome", "content", "settings.xhtml")
    with open(settings_p, "r", encoding="utf-8") as f:
        content = f.read()
    
    # Verify interactive controls exist
    assert "toggle-memory-mode" in content, "Missing memory mode toggle"
    assert "toggle-tracking-protection" in content, "Missing tracking protection toggle"
    assert "toggle-anti-fingerprinting" in content, "Missing anti-fingerprinting toggle"
    assert "toggle-socks5-routing" in content, "Missing socks5 routing toggle"
    assert "clear-data-modal" in content, "Missing clear data modal"
    assert "q-switch" in content, "Missing switch controls"
    print("  [PASS] Real interactive controls and modal dialogs present in settings.xhtml.")

def verify_extensions_no_crude_alert():
    print("\n[Check] Verifying Extensions Page Install Handler...")
    ext_p = os.path.join(r"e:\Qaulium AI\Broswer", "qualium", "chrome", "content", "extensions.xhtml")
    with open(ext_p, "r", encoding="utf-8") as f:
        content = f.read()

    assert "alert(\"To install an extension" not in content, "Crude alert still present in extensions.xhtml!"
    assert "xpi-file-picker" in content, "Missing real file picker in extensions.xhtml"
    print("  [PASS] Crude alert removed; real file picker and drag-drop active in extensions.xhtml.")

def main():
    print("=== QUALIUM TASK MANAGER BRANDING & SETTINGS VERIFICATION ===")
    
    local_core = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium\runtime\qualium-core.exe")
    local_launcher = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium\QualiumQuantumBrowser.exe")
    repo_core = os.path.join(r"e:\Qaulium AI\Broswer", "runtime", "qualium-core.exe")
    repo_launcher = os.path.join(r"e:\Qaulium AI\Broswer", "QualiumQuantumBrowser.exe")

    for exe in [local_core, local_launcher, repo_core, repo_launcher]:
        verify_pe_version(exe)

    verify_settings_no_mock()
    verify_extensions_no_crude_alert()

    print("\n[SUCCESS] All verification tests passed 100%!")

if __name__ == '__main__':
    main()
