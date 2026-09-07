import os
import shutil
import sys
import win32com.client
from ctypes import windll
import ctypes

# Import replace_icon
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from replace_icon import replace_icon

REPO_ROOT = r"e:\Qaulium AI\Broswer"
GEN_DIR = os.path.join(REPO_ROOT, "scratch", "generated_icons")
NEW_ICO = os.path.join(GEN_DIR, "qualium.ico")

LOCAL_QUALIUM = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium")
LOCAL_QAULIUM = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qaulium")

def safe_copy(src, dst):
    if not os.path.exists(src):
        print(f"Source does not exist: {src}")
        return
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    shutil.copy2(src, dst)
    print(f"Copied {src} -> {dst}")

def deploy_ico_files():
    print("--- Deploying ICO Files ---")
    ico_destinations = [
        os.path.join(REPO_ROOT, "qualium.ico"),
        os.path.join(REPO_ROOT, "resources", "qualium.ico"),
        os.path.join(REPO_ROOT, "qualium", "chrome", "content", "assets", "icons", "qualium.ico"),
        os.path.join(REPO_ROOT, "runtime", "chrome", "content", "assets", "icons", "qualium.ico"),
    ]
    if os.path.exists(LOCAL_QUALIUM):
        ico_destinations.extend([
            os.path.join(LOCAL_QUALIUM, "qualium.ico"),
            os.path.join(LOCAL_QUALIUM, "resources", "qualium.ico"),
            os.path.join(LOCAL_QUALIUM, "chrome", "content", "assets", "icons", "qualium.ico"),
            os.path.join(LOCAL_QUALIUM, "runtime", "chrome", "content", "assets", "icons", "qualium.ico"),
        ])
    if os.path.exists(LOCAL_QAULIUM):
        ico_destinations.extend([
            os.path.join(LOCAL_QAULIUM, "qualium.ico"),
            os.path.join(LOCAL_QAULIUM, "resources", "qualium.ico"),
        ])

    for dst in ico_destinations:
        safe_copy(NEW_ICO, dst)

def deploy_visual_elements():
    print("--- Deploying VisualElements Assets ---")
    ve_dirs = [
        os.path.join(REPO_ROOT, "runtime", "browser", "VisualElements"),
    ]
    if os.path.exists(LOCAL_QUALIUM):
        ve_dirs.append(os.path.join(LOCAL_QUALIUM, "runtime", "browser", "VisualElements"))
    if os.path.exists(LOCAL_QAULIUM):
        ve_dirs.append(os.path.join(LOCAL_QAULIUM, "runtime", "browser", "VisualElements"))

    ve_files = ["VisualElements_150.png", "VisualElements_70.png", "PrivateBrowsing_150.png", "PrivateBrowsing_70.png"]
    for d in ve_dirs:
        os.makedirs(d, exist_ok=True)
        for vf in ve_files:
            safe_copy(os.path.join(GEN_DIR, vf), os.path.join(d, vf))

    # Also make sure qualium-core.VisualElementsManifest.xml exists
    manifest_content = """<!-- Qualium Quantum Browser Visual Elements -->
<Application xmlns:xsi='http://www.w3.org/2001/XMLSchema-instance'>
  <VisualElements
      ShowNameOnSquare150x150Logo='on'
      Square150x150Logo='browser\\VisualElements\\VisualElements_150.png'
      Square70x70Logo='browser\\VisualElements\\VisualElements_70.png'
      ForegroundText='light'
      BackgroundColor='#070b14'/>
</Application>
"""
    manifest_paths = [
        os.path.join(REPO_ROOT, "runtime", "qualium-core.VisualElementsManifest.xml"),
        os.path.join(REPO_ROOT, "runtime", "firefox.VisualElementsManifest.xml"),
    ]
    if os.path.exists(LOCAL_QUALIUM):
        manifest_paths.extend([
            os.path.join(LOCAL_QUALIUM, "runtime", "qualium-core.VisualElementsManifest.xml"),
            os.path.join(LOCAL_QUALIUM, "runtime", "firefox.VisualElementsManifest.xml"),
            os.path.join(LOCAL_QUALIUM, "QualiumQuantumBrowser.VisualElementsManifest.xml"),
        ])
    for mp in manifest_paths:
        with open(mp, "w", encoding="utf-8") as f:
            f.write(manifest_content)
        print(f"Wrote manifest to {mp}")

def deploy_browser_logos():
    print("--- Deploying UI Logos ---")
    logo_512 = os.path.join(GEN_DIR, "qaulium_logo.png")
    logo_128 = os.path.join(GEN_DIR, "qaulium_logo_128.png")

    pairs = [
        (logo_512, os.path.join(REPO_ROOT, "qualium", "chrome", "content", "qaulium_logo.png")),
        (logo_128, os.path.join(REPO_ROOT, "qualium", "chrome", "content", "qaulium_logo_128.png")),
        (logo_512, os.path.join(REPO_ROOT, "qualium", "chrome", "skin", "qaulium_logo.png")),
        (logo_512, os.path.join(REPO_ROOT, "runtime", "chrome", "content", "qaulium_logo.png")),
        (logo_128, os.path.join(REPO_ROOT, "runtime", "chrome", "content", "qaulium_logo_128.png")),
        (logo_512, os.path.join(REPO_ROOT, "runtime", "chrome", "skin", "qaulium_logo.png")),
    ]
    if os.path.exists(LOCAL_QUALIUM):
        pairs.extend([
            (logo_512, os.path.join(LOCAL_QUALIUM, "chrome", "content", "qaulium_logo.png")),
            (logo_128, os.path.join(LOCAL_QUALIUM, "chrome", "content", "qaulium_logo_128.png")),
            (logo_512, os.path.join(LOCAL_QUALIUM, "chrome", "skin", "qaulium_logo.png")),
            (logo_512, os.path.join(LOCAL_QUALIUM, "runtime", "chrome", "content", "qaulium_logo.png")),
            (logo_128, os.path.join(LOCAL_QUALIUM, "runtime", "chrome", "content", "qaulium_logo_128.png")),
            (logo_512, os.path.join(LOCAL_QUALIUM, "runtime", "chrome", "skin", "qaulium_logo.png")),
        ])

    for s, d in pairs:
        safe_copy(s, d)

def inject_pe_icons():
    print("--- Injecting PE Executable Icons ---")
    exe_targets = [
        os.path.join(REPO_ROOT, "runtime", "qualium-core.exe"),
    ]
    if os.path.exists(LOCAL_QUALIUM):
        exe_targets.append(os.path.join(LOCAL_QUALIUM, "runtime", "qualium-core.exe"))
    if os.path.exists(LOCAL_QAULIUM):
        exe_targets.append(os.path.join(LOCAL_QAULIUM, "runtime", "qualium-core.exe"))

    for exe in exe_targets:
        if os.path.exists(exe):
            try:
                replace_icon(exe, NEW_ICO)
            except Exception as e:
                print(f"Error injecting icon into {exe}: {e}")

def update_shortcuts():
    print("--- Updating Shortcuts ---")
    wscript = win32com.client.Dispatch("WScript.Shell")
    desktop = os.path.expanduser("~/Desktop")
    start_menu = os.path.expandvars(r"%APPDATA%\Microsoft\Windows\Start Menu\Programs")

    target_ico = os.path.join(LOCAL_QUALIUM, "resources", "qualium.ico") if os.path.exists(LOCAL_QUALIUM) else os.path.join(REPO_ROOT, "resources", "qualium.ico")

    # Check Desktop
    for f in os.listdir(desktop):
        if f.endswith(".lnk") and ("qualium" in f.lower() or "browser" in f.lower()):
            lnk_path = os.path.join(desktop, f)
            try:
                sc = wscript.CreateShortcut(lnk_path)
                sc.IconLocation = f"{target_ico},0"
                sc.Save()
                print(f"Updated shortcut {lnk_path} -> Icon: {sc.IconLocation}")
            except Exception as e:
                print(f"Failed to update shortcut {lnk_path}: {e}")

    # Check Start Menu
    for root, dirs, files in os.walk(start_menu):
        for f in files:
            if f.endswith(".lnk") and ("qualium" in f.lower() or "browser" in f.lower()):
                lnk_path = os.path.join(root, f)
                try:
                    sc = wscript.CreateShortcut(lnk_path)
                    sc.IconLocation = f"{target_ico},0"
                    sc.Save()
                    print(f"Updated start menu shortcut {lnk_path} -> Icon: {sc.IconLocation}")
                except Exception as e:
                    print(f"Failed to update start menu shortcut {lnk_path}: {e}")

def refresh_shell_icon_cache():
    print("--- Refreshing Windows Shell Icon Cache ---")
    # SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, NULL, NULL)
    # SHCNE_ASSOCCHANGED = 0x08000000
    # SHCNF_IDLIST = 0x0000
    try:
        windll.shell32.SHChangeNotify(0x08000000, 0x0000, None, None)
        print("Notified Windows Shell of icon changes (SHCNE_ASSOCCHANGED).")
    except Exception as e:
        print(f"Error calling SHChangeNotify: {e}")

def main():
    deploy_ico_files()
    deploy_visual_elements()
    deploy_browser_logos()
    inject_pe_icons()
    update_shortcuts()
    refresh_shell_icon_cache()
    print("Deployment completed successfully!")

if __name__ == '__main__':
    main()
