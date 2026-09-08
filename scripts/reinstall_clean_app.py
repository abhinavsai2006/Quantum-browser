import os
import sys
import shutil
import time
import subprocess
import winreg
import win32com.client
from ctypes import windll

REPO_ROOT = r"e:\Qaulium AI\Broswer"
LOCAL_QUALIUM = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium")
LOCAL_QAULIUM = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qaulium")

DESKTOP_DIR = os.path.expanduser("~/Desktop")
START_MENU_DIR = os.path.expandvars(r"%APPDATA%\Microsoft\Windows\Start Menu\Programs")

def kill_running_processes():
    print("[1/6] Terminating any running browser or daemon processes...")
    try:
        subprocess.run([
            "powershell", "-Command",
            "Get-Process *qualium*,*qaulium*,*firefox*,*tor-real* -ErrorAction SilentlyContinue | Stop-Process -Force"
        ], check=False)
    except Exception as e:
        print(f"  Warning killing processes: {e}")
    time.sleep(1.0)

def remove_shortcuts():
    print("[2/6] Removing existing desktop and start menu shortcuts...")
    # Desktop
    for name in ["Quantum Browser.lnk", "Qualium Quantum Browser.lnk"]:
        p = os.path.join(DESKTOP_DIR, name)
        if os.path.exists(p):
            try:
                os.remove(p)
                print(f"  Removed desktop shortcut: {p}")
            except Exception as e:
                print(f"  Failed to remove {p}: {e}")

    # Start Menu
    for folder_name in ["Qualium", "Qaulium"]:
        sm_folder = os.path.join(START_MENU_DIR, folder_name)
        if os.path.exists(sm_folder):
            try:
                shutil.rmtree(sm_folder)
                print(f"  Removed start menu folder: {sm_folder}")
            except Exception as e:
                print(f"  Failed to remove {sm_folder}: {e}")

def unregister_registry():
    print("[3/6] Cleaning Windows registry uninstall entries...")
    uninstall_base = r"Software\Microsoft\Windows\CurrentVersion\Uninstall"
    for key_name in ["QualiumQuantumBrowser", "QauliumQuantumBrowser"]:
        try:
            full_key = f"{uninstall_base}\\{key_name}"
            winreg.DeleteKey(winreg.HKEY_CURRENT_USER, full_key)
            print(f"  Removed registry key: HKCU\\{full_key}")
        except FileNotFoundError:
            pass
        except Exception as e:
            print(f"  Note on registry key {key_name}: {e}")

def delete_installed_directories():
    print("[4/6] Removing installed program directories...")
    for d in [LOCAL_QUALIUM, LOCAL_QAULIUM]:
        if os.path.exists(d):
            try:
                shutil.rmtree(d)
                print(f"  Deleted installed directory: {d}")
            except Exception as e:
                print(f"  Warning deleting {d}: {e}")
                # Try retrying after a brief pause
                time.sleep(1.0)
                try:
                    shutil.rmtree(d, ignore_errors=True)
                except Exception:
                    pass

    # Clean profile lock files, stale sessions and startup cache so installation opens completely fresh
    for prof in [r"%LOCALAPPDATA%\Qaulium\Profile", r"%LOCALAPPDATA%\Qualium\Profile"]:
        pdir = os.path.expandvars(prof)
        for item in ["parent.lock", ".parentlock", "sessionstore.jsonlz4", "sessionstore.js"]:
            p = os.path.join(pdir, item)
            if os.path.exists(p):
                try:
                    os.remove(p)
                    print(f"  Removed profile lock/session: {p}")
                except Exception:
                    pass
        for sdir in ["sessionstore-backups", "startupCache"]:
            p = os.path.join(pdir, sdir)
            if os.path.exists(p):
                try:
                    shutil.rmtree(p, ignore_errors=True)
                    print(f"  Purged cache directory: {p}")
                except Exception:
                    pass

def perform_fresh_installation():
    print("[5/6] Performing fresh installation into system...")
    dest = LOCAL_QUALIUM
    os.makedirs(dest, exist_ok=True)

    # 1. Copy primary binaries
    browser_src = os.path.join(REPO_ROOT, "target", "release", "QualiumQuantumBrowser.exe")
    if os.path.exists(browser_src):
        shutil.copy2(browser_src, os.path.join(REPO_ROOT, "QualiumQuantumBrowser.exe"))
    
    shutil.copy2(os.path.join(REPO_ROOT, "QualiumQuantumBrowser.exe"), os.path.join(dest, "QualiumQuantumBrowser.exe"))
    shutil.copy2(os.path.join(REPO_ROOT, "QualiumQuantumBrowser.exe"), os.path.join(dest, "QauliumQuantumBrowser.exe"))

    # Daemon
    daemon_src = os.path.join(REPO_ROOT, "target", "release", "qualium-daemon.exe")
    if os.path.exists(daemon_src):
        shutil.copy2(daemon_src, os.path.join(REPO_ROOT, "qualium-daemon.exe"))
    elif not os.path.exists(daemon_src):
        daemon_src = os.path.join(REPO_ROOT, "dist", "qualium-daemon.exe")
    if os.path.exists(daemon_src):
        shutil.copy2(daemon_src, os.path.join(dest, "qualium-daemon.exe"))
        print("  Copied qualium-daemon.exe")

    # Uninstaller
    uninstaller_src = os.path.join(REPO_ROOT, "dist", "QualiumUninstall.exe")
    if os.path.exists(uninstaller_src):
        shutil.copy2(uninstaller_src, os.path.join(dest, "QualiumUninstall.exe"))
        shutil.copy2(uninstaller_src, os.path.join(dest, "QauliumUninstall.exe"))
        os.makedirs(os.path.join(dest, "uninstall"), exist_ok=True)
        shutil.copy2(uninstaller_src, os.path.join(dest, "uninstall", "helper.exe"))

    # 2. Copy resources & icons
    res_dir = os.path.join(dest, "resources")
    os.makedirs(res_dir, exist_ok=True)
    shutil.copy2(os.path.join(REPO_ROOT, "resources", "qualium.ico"), os.path.join(res_dir, "qualium.ico"))
    shutil.copy2(os.path.join(REPO_ROOT, "resources", "qualium.ico"), os.path.join(dest, "qualium.ico"))

    # 3. Copy full runtime directory
    runtime_dst = os.path.join(dest, "runtime")
    shutil.copytree(os.path.join(REPO_ROOT, "runtime"), runtime_dst, dirs_exist_ok=True)
    print("  Copied runtime engine")

    # 4. Copy chrome and manifests
    chrome_dst = os.path.join(dest, "chrome")
    shutil.copytree(os.path.join(REPO_ROOT, "qualium", "chrome"), chrome_dst, dirs_exist_ok=True)

    for f in ["chrome.manifest", "application.ini", "QualiumQuantumBrowser.VisualElementsManifest.xml"]:
        src_f = os.path.join(REPO_ROOT, f)
        if os.path.exists(src_f):
            shutil.copy2(src_f, os.path.join(dest, f))

    # Manifests
    manifest_content = """<!-- Quantum Browser Visual Elements -->
<Application xmlns:xsi='http://www.w3.org/2001/XMLSchema-instance'>
  <VisualElements
      ShowNameOnSquare150x150Logo='on'
      Square150x150Logo='browser\\VisualElements\\VisualElements_150.png'
      Square70x70Logo='browser\\VisualElements\\VisualElements_70.png'
      ForegroundText='light'
      BackgroundColor='#070b14'/>
</Application>
"""
    for mf in ["QualiumQuantumBrowser.VisualElementsManifest.xml", "QauliumQuantumBrowser.VisualElementsManifest.xml"]:
        with open(os.path.join(dest, mf), "w", encoding="utf-8") as fl:
            fl.write(manifest_content)

    # Also replicate to %LOCALAPPDATA%\Programs\Qaulium for clean dual-compatibility
    try:
        shutil.copytree(dest, LOCAL_QAULIUM, dirs_exist_ok=True)
        print(f"  Mirrored to {LOCAL_QAULIUM}")
    except Exception as e:
        print(f"  Mirroring notice: {e}")

    # 5. Inject PE version info and icons into installed executables
    sys.path.insert(0, os.path.join(REPO_ROOT, "scripts"))
    try:
        import inject_all_version_info
        inject_all_version_info.main()
        import replace_icon
        replace_icon.replace_icon(os.path.join(dest, "runtime", "qualium-core.exe"), os.path.join(res_dir, "qualium.ico"))
        print("  Re-injected PE icons and version info into installed binaries")
    except Exception as e:
        print(f"  Notice during PE injection: {e}")

def create_shortcuts_and_register():
    print("[6/6] Creating shortcuts and registering with Windows...")
    wscript = win32com.client.Dispatch("WScript.Shell")
    dest = LOCAL_QUALIUM
    exe_target = os.path.join(dest, "QualiumQuantumBrowser.exe")
    ico_target = os.path.join(dest, "resources", "qualium.ico")

    # Desktop shortcut
    desktop_lnk = os.path.join(DESKTOP_DIR, "Quantum Browser.lnk")
    sc = wscript.CreateShortcut(desktop_lnk)
    sc.TargetPath = exe_target
    sc.WorkingDirectory = dest
    sc.IconLocation = f"{ico_target},0"
    sc.Description = "Quantum Browser — Next-Gen Post-Quantum Privacy Web Browser"
    sc.Save()
    print(f"  Created Desktop shortcut: {desktop_lnk}")

    # Start Menu shortcuts
    sm_qaulium = os.path.join(START_MENU_DIR, "Qaulium")
    os.makedirs(sm_qaulium, exist_ok=True)
    sm_lnk = os.path.join(sm_qaulium, "Quantum Browser.lnk")
    sc_sm = wscript.CreateShortcut(sm_lnk)
    sc_sm.TargetPath = exe_target
    sc_sm.WorkingDirectory = dest
    sc_sm.IconLocation = f"{ico_target},0"
    sc_sm.Description = "Quantum Browser"
    sc_sm.Save()
    print(f"  Created Start Menu shortcut: {sm_lnk}")

    # Uninstall shortcut
    uninst_lnk = os.path.join(sm_qaulium, "Uninstall Quantum Browser.lnk")
    sc_un = wscript.CreateShortcut(uninst_lnk)
    sc_un.TargetPath = os.path.join(dest, "QualiumUninstall.exe")
    sc_un.WorkingDirectory = dest
    sc_un.IconLocation = f"{ico_target},0"
    sc_un.Description = "Uninstall Quantum Browser"
    sc_un.Save()
    print(f"  Created Uninstall shortcut: {uninst_lnk}")

    # Windows Registry Registration
    reg_path = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\QauliumQuantumBrowser"
    try:
        with winreg.CreateKey(winreg.HKEY_CURRENT_USER, reg_path) as k:
            winreg.SetValueEx(k, "DisplayName", 0, winreg.REG_SZ, "Quantum Browser")
            winreg.SetValueEx(k, "DisplayVersion", 0, winreg.REG_SZ, "5.0.0")
            winreg.SetValueEx(k, "Publisher", 0, winreg.REG_SZ, "Quantum Browser Project")
            winreg.SetValueEx(k, "DisplayIcon", 0, winreg.REG_SZ, ico_target)
            winreg.SetValueEx(k, "InstallLocation", 0, winreg.REG_SZ, dest)
            winreg.SetValueEx(k, "UninstallString", 0, winreg.REG_SZ, f'"{os.path.join(dest, "QualiumUninstall.exe")}"')
            winreg.SetValueEx(k, "QuietUninstallString", 0, winreg.REG_SZ, f'"{os.path.join(dest, "QualiumUninstall.exe")}" /S')
            winreg.SetValueEx(k, "URLInfoAbout", 0, winreg.REG_SZ, "https://quantumbrowser.org")
        print("  Registered Quantum Browser in Windows Installed Apps registry.")
    except Exception as e:
        print(f"  Registry registration note: {e}")

    # Flush Windows Shell & Icon Cache
    windll.shell32.SHChangeNotify(0x08000000, 0x0000, None, None)
    print("  Flushed Windows shell icon cache.")

def main():
    print("=" * 70)
    print("QUANTUM BROWSER — COMPLETE RE-INSTALLATION PIPELINE")
    print("=" * 70)
    kill_running_processes()
    remove_shortcuts()
    unregister_registry()
    delete_installed_directories()
    perform_fresh_installation()
    create_shortcuts_and_register()
    print("\n[SUCCESS] Quantum Browser removed and cleanly reinstalled on your system!")

if __name__ == "__main__":
    main()
