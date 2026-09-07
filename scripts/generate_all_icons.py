import os
import struct
from PIL import Image, ImageFilter
import numpy as np
from collections import deque

def create_transparent_master(source_jpg_path):
    print(f"Reading source image from {source_jpg_path}...")
    im = Image.open(source_jpg_path).convert('RGB')
    arr = np.array(im)
    h, w, _ = arr.shape

    # Flood fill from corners to find outside dark background
    visited = np.zeros((h, w), dtype=bool)
    q = deque([(0, 0), (0, w - 1), (h - 1, 0), (h - 1, w - 1)])
    for pt in q:
        visited[pt] = True

    brightness = np.max(arr, axis=2)

    while q:
        y, x = q.popleft()
        for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)):
            ny, nx = y + dy, x + dx
            if 0 <= ny < h and 0 <= nx < w:
                if not visited[ny, nx] and brightness[ny, nx] < 12:
                    visited[ny, nx] = True
                    q.append((ny, nx))

    badge_mask = (~visited).astype(np.uint8) * 255
    mask_img = Image.fromarray(badge_mask)
    # 0.8px gaussian blur for soft anti-aliased edges
    mask_img = mask_img.filter(ImageFilter.GaussianBlur(radius=0.8))

    rgba = im.convert('RGBA')
    rgba.putalpha(mask_img)

    # Crop precisely around the squircle with balanced padding
    bbox = (128 - 8, 126 - 8, 895 + 8, 895 + 8)
    cropped = rgba.crop(bbox)

    # Place in 1024x1024 square canvas with ~7.5% margins (modern standard)
    target_dim = 904
    cw, ch = cropped.size
    scale = target_dim / max(cw, ch)
    new_w = int(cw * scale)
    new_h = int(ch * scale)
    resized = cropped.resize((new_w, new_h), Image.Resampling.LANCZOS)

    canvas_1024 = Image.new('RGBA', (1024, 1024), (0, 0, 0, 0))
    offset_x = (1024 - new_w) // 2
    offset_y = (1024 - new_h) // 2
    canvas_1024.paste(resized, (offset_x, offset_y), resized)
    
    return canvas_1024

def save_ico_custom(img_1024, ico_path, sizes=(256, 128, 64, 48, 32, 24, 16)):
    """
    Saves a high quality multi-resolution Windows ICO file with 32-bit RGBA PNG sub-images.
    """
    import io
    images_data = []
    
    for s in sizes:
        resized = img_1024.resize((s, s), Image.Resampling.LANCZOS)
        buf = io.BytesIO()
        resized.save(buf, format='PNG', optimize=True)
        images_data.append((s, buf.getvalue()))

    # Build ICO file
    count = len(images_data)
    # Header: reserved (0), type (1 for ico), count
    ico_bytes = bytearray(struct.pack('<HHH', 0, 1, count))
    
    offset = 6 + (16 * count)
    for s, data in images_data:
        w_byte = 0 if s >= 256 else s
        h_byte = 0 if s >= 256 else s
        # ICONDIRENTRY: width, height, num_colors, reserved, planes, bpp, bytes_in_res, image_offset
        ico_bytes.extend(struct.pack('<BBBBHHII', w_byte, h_byte, 0, 0, 1, 32, len(data), offset))
        offset += len(data)
        
    for _, data in images_data:
        ico_bytes.extend(data)
        
    os.makedirs(os.path.dirname(os.path.abspath(ico_path)), exist_ok=True)
    with open(ico_path, 'wb') as f:
        f.write(ico_bytes)
    print(f"Saved ICO ({len(sizes)} resolutions, {len(ico_bytes)} bytes) to {ico_path}")

def main():
    src_jpg = r"C:\Users\mndab\.gemini\antigravity-ide\brain\b0a47078-44ea-43d7-ae34-0240dd9ce43a\qualium_browser_app_icon_1788767191732.jpg"
    master = create_transparent_master(src_jpg)
    
    os.makedirs("scratch/generated_icons", exist_ok=True)
    master_1024_path = "scratch/generated_icons/qualium_app_icon_1024.png"
    master.save(master_1024_path)
    print("Saved master 1024x1024 PNG.")

    # Save ICO with full multi-resolution suite
    save_ico_custom(master, "scratch/generated_icons/qualium.ico", sizes=(256, 128, 64, 48, 32, 24, 16))

    # Generate PNG sizes needed for Windows VisualElements, branding, and browser UI
    png_specs = {
        "qaulium_logo.png": (512, 512),
        "qaulium_logo_128.png": (128, 128),
        "VisualElements_150.png": (150, 150),
        "VisualElements_70.png": (70, 70),
        "PrivateBrowsing_150.png": (150, 150),
        "PrivateBrowsing_70.png": (70, 70),
        "icon16.png": (16, 16),
        "icon32.png": (32, 32),
        "icon48.png": (48, 48),
        "icon64.png": (64, 64),
        "icon128.png": (128, 128),
        "about-logo.png": (192, 192),
        "about-logo@2x.png": (384, 384),
        "about-logo-private.png": (192, 192),
        "about-logo-private@2x.png": (384, 384),
    }

    for name, (w, h) in png_specs.items():
        out_p = os.path.join("scratch/generated_icons", name)
        resized = master.resize((w, h), Image.Resampling.LANCZOS)
        resized.save(out_p, optimize=True)
        print(f"Generated {name} ({w}x{h})")

if __name__ == '__main__':
    main()
