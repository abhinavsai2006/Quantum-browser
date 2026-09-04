import zipfile, io, struct, os, sys

def deoptimize_jar(data):
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

def optimize_jar(std_zip_data, first_4_bytes=None):
    eocd_idx = std_zip_data.rfind(b'PK\x05\x06')
    cd_offset = struct.unpack('<I', std_zip_data[eocd_idx+16:eocd_idx+20])[0]
    local_part = std_zip_data[:cd_offset]
    cd_part = bytearray(std_zip_data[cd_offset:eocd_idx])
    eocd_part = bytearray(std_zip_data[eocd_idx:eocd_idx+22])
    
    shift = 4 + len(cd_part) + len(eocd_part)
    idx = 0
    while idx < len(cd_part):
        if cd_part[idx:idx+4] == b'PK\x01\x02':
            orig_off = struct.unpack('<I', cd_part[idx+42:idx+46])[0]
            struct.pack_into('<I', cd_part, idx+42, orig_off + shift)
            name_len = struct.unpack('<H', cd_part[idx+28:idx+30])[0]
            extra_len = struct.unpack('<H', cd_part[idx+30:idx+32])[0]
            comm_len = struct.unpack('<H', cd_part[idx+32:idx+34])[0]
            idx += 46 + name_len + extra_len + comm_len
        else: break
    
    # In optimized jar, EOCD offset of CD is 4 (since CD starts at byte 4)
    struct.pack_into('<I', eocd_part, 16, 4)
    
    if first_4_bytes is None:
        cd_len = len(cd_part) + len(eocd_part)
        first_4_bytes = struct.pack('<I', cd_len)
        
    return first_4_bytes + cd_part + eocd_part + local_part

print("Testing deoptimize and optimize functions...")
omni_path = r"runtime/browser/omni.ja"
with open(omni_path, "rb") as f:
    raw = f.read()

hdr4 = raw[:4]
std_data = deoptimize_jar(raw)
opt_data = optimize_jar(std_data, hdr4)

diff_count = sum(1 for i in range(len(raw)) if raw[i] != opt_data[i])
print(f"Total diff bytes after round-trip: {diff_count} (out of {len(raw)})")
