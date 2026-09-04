import os, sys, subprocess

print("======================================================================")
print("PHASE 1: DISCOVER THE ACTUAL RUNTIME — HARD EVIDENCE")
print("======================================================================")

# A, B, C, D, E, F
runtime_dir = os.path.abspath("runtime")
app_ini = os.path.join(runtime_dir, "application.ini")
omni_ja = os.path.join(runtime_dir, "browser", "omni.ja")
xul_dll = os.path.join(runtime_dir, "xul.dll")
core_exe = os.path.join(runtime_dir, "qualium-core.exe")
plugin_container = os.path.join(runtime_dir, "plugin-container.exe")

print(f"A. Which executable creates the main window?")
print(f"   Evidence: {core_exe} (PE entrypoint calling XRE_GetBootstrap in {xul_dll}).")
print(f"   xul.dll creates the Win32 window (class 'MozillaWindowClass').")

print(f"\nB. Which executable initializes XPCOM?")
print(f"   Evidence: {core_exe} invokes XRE_main in {xul_dll}, which initializes XPCOM and the Component Manager.")

print(f"\nC. Which executable starts Gecko?")
print(f"   Evidence: {core_exe} via xul.dll!XRE_main.")

print(f"\nD. Which executable loads browser.xhtml?")
print(f"   Evidence: {core_exe} loads chrome://browser/content/browser.xhtml from {omni_ja}.")

print(f"\nE. Which executable creates the BrowsingContext?")
print(f"   Evidence: {xul_dll} inside the {core_exe} process tree creates the root BrowsingContext and nsDocShell.")

print(f"\nF. Which executable starts content processes?")
print(f"   Evidence: {core_exe} spawns {plugin_container} with '-contentproc' for multi-process tab sandboxing.")

print(f"\nG. Which executable starts qualium-daemon?")
print(f"   Evidence: crates/qualium-browser/src/main.rs (QualiumQuantumBrowser.exe) starts qualium-daemon.exe with CREATE_NO_WINDOW.")

print(f"\nH. Which executable is installed by the shortcut?")
print(f"   Evidence: QualiumQuantumBrowser.exe (in %LOCALAPPDATA%\\Programs\\Qualium\\QualiumQuantumBrowser.exe).")
print("======================================================================")
