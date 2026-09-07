import os
import sys
import struct
import win32api
import win32con

def pad4(data):
    rem = len(data) % 4
    if rem != 0:
        return data + b'\x00' * (4 - rem)
    return data

def make_string_struct(key_str, val_str):
    key_bytes = key_str.encode('utf-16le') + b'\x00\x00'
    val_bytes = val_str.encode('utf-16le') + b'\x00\x00'
    val_len_wchars = len(val_bytes) // 2
    
    header = struct.pack('<HHH', 0, val_len_wchars, 1) # wLength, wValueLength, wType (1=text)
    data = pad4(header + key_bytes) + val_bytes
    wLength = len(data)
    return struct.pack('<H', wLength) + data[2:]

def make_string_table(table_lang_key, strings_dict):
    key_bytes = table_lang_key.encode('utf-16le') + b'\x00\x00'
    children_bytes = b''
    for k, v in strings_dict.items():
        children_bytes = pad4(children_bytes + make_string_struct(k, v))
        
    header = struct.pack('<HHH', 0, 0, 1) # wLength, wValueLength=0, wType=1
    data = pad4(header + key_bytes) + children_bytes
    wLength = len(data)
    return struct.pack('<H', wLength) + data[2:]

def make_string_file_info(tables_list):
    key_bytes = 'StringFileInfo'.encode('utf-16le') + b'\x00\x00'
    children_bytes = b''
    for tbl in tables_list:
        children_bytes = pad4(children_bytes + tbl)
        
    header = struct.pack('<HHH', 0, 0, 1)
    data = pad4(header + key_bytes) + children_bytes
    wLength = len(data)
    return struct.pack('<H', wLength) + data[2:]

def make_var_file_info(translations):
    key_bytes = 'VarFileInfo'.encode('utf-16le') + b'\x00\x00'
    var_key = 'Translation'.encode('utf-16le') + b'\x00\x00'
    
    val_bytes = b''
    for lang, cp in translations:
        val_bytes += struct.pack('<HH', lang, cp)
        
    var_header = struct.pack('<HHH', 0, len(val_bytes), 0) # wType=0 (binary)
    var_data = pad4(var_header + var_key) + val_bytes
    var_data = struct.pack('<H', len(var_data)) + var_data[2:]
    
    header = struct.pack('<HHH', 0, 0, 1)
    data = pad4(header + key_bytes) + pad4(var_data)
    wLength = len(data)
    return struct.pack('<H', wLength) + data[2:]

def make_version_info(strings_dict, version=(5, 0, 0, 0)):
    dwSignature = 0xFEEF04BD
    dwStrucVersion = 0x00010000
    dwFileVersionMS = (version[0] << 16) | version[1]
    dwFileVersionLS = (version[2] << 16) | version[3]
    dwProductVersionMS = dwFileVersionMS
    dwProductVersionLS = dwFileVersionLS
    dwFileFlagsMask = 0x3F
    dwFileFlags = 0
    dwFileOS = 0x00040004 # VOS_NT_WINDOWS32
    dwFileType = 0x00000001 # VFT_APP
    dwFileSubtype = 0
    dwFileDateMS = 0
    dwFileDateLS = 0
    
    fixed_info = struct.pack('<IIIIIIIIIIIII',
        dwSignature, dwStrucVersion,
        dwFileVersionMS, dwFileVersionLS,
        dwProductVersionMS, dwProductVersionLS,
        dwFileFlagsMask, dwFileFlags,
        dwFileOS, dwFileType, dwFileSubtype,
        dwFileDateMS, dwFileDateLS
    )
    
    tbl_0000 = make_string_table('000004b0', strings_dict)
    tbl_0409 = make_string_table('040904b0', strings_dict)
    str_info = make_string_file_info([tbl_0000, tbl_0409])
    
    var_info = make_var_file_info([(0x0000, 0x04b0), (0x0409, 0x04b0)])
    
    root_key = 'VS_VERSION_INFO'.encode('utf-16le') + b'\x00\x00'
    root_header = struct.pack('<HHH', 0, len(fixed_info), 0)
    
    body = pad4(root_header + root_key) + pad4(fixed_info) + pad4(str_info) + pad4(var_info)
    wLength = len(body)
    return struct.pack('<H', wLength) + body[2:]

def inject_version_info(exe_path, strings_dict, version=(5, 0, 0, 0)):
    if not os.path.exists(exe_path):
        return False
    res_bytes = make_version_info(strings_dict, version)
    h_update = win32api.BeginUpdateResource(exe_path, False)
    if not h_update:
        print(f"Cannot BeginUpdateResource for {exe_path}")
        return False
    try:
        win32api.UpdateResource(h_update, win32con.RT_VERSION, 1, res_bytes, 1033)
        win32api.UpdateResource(h_update, win32con.RT_VERSION, 1, res_bytes, 0)
        win32api.EndUpdateResource(h_update, False)
        print(f"Successfully injected RT_VERSION into {exe_path}")
        return True
    except Exception as e:
        win32api.EndUpdateResource(h_update, True)
        print(f"Failed to inject RT_VERSION into {exe_path}: {e}")
        return False

def main():
    repo_root = r"e:\Qaulium AI\Broswer"
    local_app = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qualium")
    local_app_qaulium = os.path.expandvars(r"%LOCALAPPDATA%\Programs\Qaulium")

    core_strings = {
        'Comments': 'Qualium Quantum Browser with Post-Quantum Security',
        'CompanyName': 'Qaulium AI',
        'FileDescription': 'Qualium Quantum Browser',
        'FileVersion': '5.0.0.0',
        'InternalName': 'qualium-core',
        'LegalCopyright': 'Copyright © 2026 Qaulium AI. All rights reserved.',
        'OriginalFilename': 'qualium-core.exe',
        'ProductName': 'Qualium Quantum Browser',
        'ProductVersion': '5.0.0.0'
    }

    launcher_strings = {
        'Comments': 'Qualium Quantum Browser Launcher',
        'CompanyName': 'Qaulium AI',
        'FileDescription': 'Qualium Quantum Browser',
        'FileVersion': '5.0.0.0',
        'InternalName': 'QualiumQuantumBrowser',
        'LegalCopyright': 'Copyright © 2026 Qaulium AI. All rights reserved.',
        'OriginalFilename': 'QualiumQuantumBrowser.exe',
        'ProductName': 'Qualium Quantum Browser',
        'ProductVersion': '5.0.0.0'
    }

    sandbox_strings = {
        'Comments': 'Qualium Quantum Content Sandbox Process',
        'CompanyName': 'Qaulium AI',
        'FileDescription': 'Qualium Quantum Tab Sandbox',
        'FileVersion': '5.0.0.0',
        'InternalName': 'plugin-container',
        'LegalCopyright': 'Copyright © 2026 Qaulium AI. All rights reserved.',
        'OriginalFilename': 'plugin-container.exe',
        'ProductName': 'Qualium Quantum Browser',
        'ProductVersion': '5.0.0.0'
    }

    daemon_strings = {
        'Comments': 'Qaulium Quantum Security & Onion Routing Daemon',
        'CompanyName': 'Qaulium AI',
        'FileDescription': 'Qaulium Security Engine',
        'FileVersion': '5.0.0.0',
        'InternalName': 'qualium-daemon',
        'LegalCopyright': 'Copyright © 2026 Qaulium AI. All rights reserved.',
        'OriginalFilename': 'qualium-daemon.exe',
        'ProductName': 'Qualium Quantum Browser',
        'ProductVersion': '5.0.0.0'
    }

    targets = [
        # Core engines
        (os.path.join(repo_root, "runtime", "qualium-core.exe"), core_strings),
        (os.path.join(local_app, "runtime", "qualium-core.exe"), core_strings),
        (os.path.join(local_app_qaulium, "runtime", "qualium-core.exe"), core_strings),

        # Launchers
        (os.path.join(repo_root, "QualiumQuantumBrowser.exe"), launcher_strings),
        (os.path.join(local_app, "QualiumQuantumBrowser.exe"), launcher_strings),
        (os.path.join(local_app, "QauliumQuantumBrowser.exe"), launcher_strings),
        (os.path.join(local_app_qaulium, "QualiumQuantumBrowser.exe"), launcher_strings),
        (os.path.join(local_app_qaulium, "QauliumQuantumBrowser.exe"), launcher_strings),

        # Content sandboxes
        (os.path.join(repo_root, "runtime", "plugin-container.exe"), sandbox_strings),
        (os.path.join(local_app, "runtime", "plugin-container.exe"), sandbox_strings),

        # Security Daemons
        (os.path.join(local_app, "qualium-daemon.exe"), daemon_strings),
        (os.path.join(local_app_qaulium, "qualium-daemon.exe"), daemon_strings),
    ]

    for exe, strings in targets:
        inject_version_info(exe, strings)

    # Also update updater.ini in all locations
    updater_ini_content = """; Qualium Update Configuration
[Strings]
Title=Qualium Update
Info=Qualium Quantum Browser is installing updates and will start in a few moments…
MozillaMaintenanceDescription=The Qualium Maintenance Service ensures that you have the latest and most secure version of Qualium Quantum Browser on your computer.

[PostUpdateWin]
ExeRelPath=uninstall\\helper.exe
ExeArg=/PostUpdate
"""
    for u_path in [
        os.path.join(repo_root, "runtime", "updater.ini"),
        os.path.join(local_app, "runtime", "updater.ini"),
        os.path.join(local_app_qaulium, "runtime", "updater.ini"),
    ]:
        if os.path.exists(os.path.dirname(u_path)):
            with open(u_path, "w", encoding="utf-8") as f:
                f.write(updater_ini_content)
            print(f"Updated {u_path}")

if __name__ == '__main__':
    main()
