import struct

with open(r'runtime/xul.dll', 'rb') as f:
    data = f.read(4096)
    pe_offset = struct.unpack_from('<I', data, 0x3c)[0]
    f.seek(pe_offset)
    pe_header = f.read(264)
    # Magic
    magic = struct.unpack_from('<H', pe_header, 24)[0]
    is_64 = (magic == 0x20b)
    export_rva_offset = 24 + (112 if is_64 else 96)
    export_rva = struct.unpack_from('<I', pe_header, export_rva_offset)[0]
    export_size = struct.unpack_from('<I', pe_header, export_rva_offset + 4)[0]

    # Section headers
    num_sections = struct.unpack_from('<H', pe_header, 6)[0]
    opt_header_size = struct.unpack_from('<H', pe_header, 20)[0]
    f.seek(pe_offset + 24 + opt_header_size)

    sections = []
    for _ in range(num_sections):
        sec = f.read(40)
        vsize, va, rsize, rptr = struct.unpack_from('<IIII', sec, 8)
        sections.append((va, va + vsize, rptr))

    def rva_to_offset(rva):
        for start, end, rptr in sections:
            if start <= rva < end:
                return rptr + (rva - start)
        return None

    exp_offset = rva_to_offset(export_rva)
    if exp_offset:
        f.seek(exp_offset)
        exp_dir = f.read(40)
        num_names = struct.unpack_from('<I', exp_dir, 24)[0]
        names_rva = struct.unpack_from('<I', exp_dir, 32)[0]
        f.seek(rva_to_offset(names_rva))
        name_rvas = [struct.unpack('<I', f.read(4))[0] for _ in range(num_names)]

        names = []
        for nrva in name_rvas:
            noff = rva_to_offset(nrva)
            if noff:
                f.seek(noff)
                chars = []
                while True:
                    b = f.read(1)
                    if not b or b == b'\0': break
                    chars.append(b)
                names.append(b''.join(chars).decode('utf-8', 'ignore'))

        print(f'Total exports found: {len(names)}')
        for n in names:
            if 'XRE' in n or 'main' in n.lower() or 'gecko' in n.lower():
                print(' -', n)
