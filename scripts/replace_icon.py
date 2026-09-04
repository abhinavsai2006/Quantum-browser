import struct
import win32api
import win32con
import sys

def replace_icon(exe_path, ico_path):
    with open(ico_path, 'rb') as f:
        ico_data = f.read()

    # Parse ICO header
    reserved, ico_type, count = struct.unpack('<HHH', ico_data[:6])
    if ico_type != 1:
        raise ValueError("Not a valid ICO file")

    dir_entries = []
    icon_images = []
    offset = 6
    for i in range(count):
        width, height, colors, reserved, planes, bpp, size, img_offset = struct.unpack('<BBBBHHII', ico_data[offset:offset+16])
        dir_entries.append((width, height, colors, reserved, planes, bpp, size, i + 1))
        icon_images.append(ico_data[img_offset:img_offset+size])
        offset += 16

    # Build GRPICONDIR structure
    grp_dir = struct.pack('<HHH', 0, 1, count)
    for entry in dir_entries:
        # GRPICONDIRENTRY: width, height, colors, reserved, planes, bpp, bytes_in_res, id
        grp_dir += struct.pack('<BBBBHHIH', entry[0], entry[1], entry[2], entry[3], entry[4], entry[5], entry[6], entry[7])

    # Begin resource update
    h_update = win32api.BeginUpdateResource(exe_path, False)
    if not h_update:
        raise RuntimeError("Failed to open executable for resource update")

    try:
        # Update each RT_ICON (Type 3)
        for i, img in enumerate(icon_images):
            win32api.UpdateResource(h_update, win32con.RT_ICON, i + 1, img, 1033)

        # Update RT_GROUP_ICON (Type 14, ID 1 or 32512)
        # 1 is standard primary application icon resource ID
        win32api.UpdateResource(h_update, win32con.RT_GROUP_ICON, 1, grp_dir, 1033)
        win32api.UpdateResource(h_update, win32con.RT_GROUP_ICON, 32512, grp_dir, 1033)
        win32api.EndUpdateResource(h_update, False)
        print(f"Successfully injected qualium icon into {exe_path}")
    except Exception as e:
        win32api.EndUpdateResource(h_update, True)
        raise e

if __name__ == '__main__':
    exe = sys.argv[1]
    ico = sys.argv[2]
    replace_icon(exe, ico)
