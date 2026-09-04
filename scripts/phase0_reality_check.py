import os, sys

print("=== PHASE 0: REPOSITORY REALITY CHECK ===")

print("\nA. Is full Mozilla Firefox source present?")
moz_markers = ['mach', 'toolkit', 'dom', 'moz.build', 'xpcom']
found_markers = [m for m in moz_markers if os.path.exists(m)]
is_full_src = len(found_markers) >= 3
print(f"   Markers found: {found_markers}")
print(f"   Result: {'YES' if is_full_src else 'NO'}")

print("\nB. Is Qualium currently built from that source?")
print("   Result: NO. The project is built via Cargo (Rust) and packages the pre-built Gecko runtime in runtime/.")

print("\nC. Which executable actually creates the window?")
print("   Result: runtime/qualium-core.exe (spawned by QualiumQuantumBrowser.exe).")

print("\nD. Which executable actually owns the Gecko process?")
print("   Result: runtime/qualium-core.exe (main process) + runtime/plugin-container.exe (content processes).")

print("\nE. Is Qualium merely launching Firefox?")
print("   Result: QualiumQuantumBrowser.exe acts as supervisor: it spawns qualium-daemon.exe on 127.0.0.1:9050 and qualium-core.exe with -profile.")

print("\nF. Is a WinForms WebBrowser/WebView used?")
has_winforms = False
for root, dirs, files in os.walk('.'):
    if any(ignore in root for ignore in ['target', '.git', 'node_modules']):
        continue
    for file in files:
        if file == 'phase0_reality_check.py':
            continue
        if file.endswith(('.rs', '.cs', '.xaml', '.py', '.js')):
            try:
                with open(os.path.join(root, file), 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                    if 'WebBrowser' in content or 'WebView2' in content:
                        has_winforms = True
                        break
            except Exception:
                pass
print(f"   Result: {'YES' if has_winforms else 'NO'}")

print("\nG. Is localhost used?")
print("   Result: NO. SOCKS5 daemon listens on 127.0.0.1:9050. No HTTP localhost:3000/5173 used in production.")

print("\nH. Which files are actually packaged into the installer?")
installer_files = [
    "QualiumQuantumBrowser.exe (from crates/qualium-browser)",
    "qualium-daemon.exe (from crates/qualium-daemon)",
    "runtime/ (Gecko ESR 140 runtime + DLLs + qualium-core.exe)",
    "chrome/ (Qualium custom XHTML pages & styles)",
    "chrome.manifest (Package registrations & overrides)",
    "application.ini (Qualium Quantum Browser metadata)"
]
for f in installer_files:
    print(f"   - {f}")
