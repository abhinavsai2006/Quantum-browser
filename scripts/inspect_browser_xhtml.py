import zipfile, io, struct

def deoptimize_jar(data):
    eocd_idx = data.find(b'PK\x05\x06')
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

with open(r'runtime/browser/omni.ja', 'rb') as f:
    zf = zipfile.ZipFile(io.BytesIO(deoptimize_jar(f.read())))

b = zf.read('chrome/browser/content/browser/browser.xhtml').decode('utf-8', 'ignore')

pos = b.find('id="appMenu-mainView"')
if pos != -1:
    end = b.find('</panelview>', pos)
    print("=== appMenu-mainView ===")
    print(b[pos:end+12])

pos_prot = b.find('id="protections-popup"')
if pos_prot != -1:
    end_prot = b.find('</panel>', pos_prot)
    print("=== protections-popup ===")
    print(b[pos_prot:pos_prot+1200])
