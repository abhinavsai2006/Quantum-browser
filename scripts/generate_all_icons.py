import os
import struct
from PIL import Image, ImageFilter
import numpy as np
from collections import deque

def create_transparent_master(source_logo_path):
    print(f"Reading old green logo from {source_logo_path}...")
    import cv2
    img_bgr = cv2.imread(source_logo_path)
    h, w, _ = img_bgr.shape

    # Find the outer contour of the crystal shield
    gray = cv2.cvtColor(img_bgr, cv2.COLOR_BGR2GRAY)
    roi_mask = np.zeros((h, w), dtype=np.uint8)
    roi_mask[200:820, 240:784] = 255
    bright = (gray > 100).astype(np.uint8) * roi_mask
    contours, _ = cv2.findContours(bright, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    shield_cnt = max(contours, key=cv2.contourArea)

    # Draw filled mask of the shield
    mask = np.zeros((h, w), dtype=np.uint8)
    cv2.drawContours(mask, [shield_cnt], -1, 255, -1)
    
    # Anti-alias mask with 3x3 gaussian blur for crisp subpixel edges
    mask_blurred = cv2.GaussianBlur(mask, (3, 3), 0)

    # Convert to RGBA
    img_rgba = cv2.cvtColor(img_bgr, cv2.COLOR_BGR2RGBA)
    img_rgba[:, :, 3] = mask_blurred

    im_pil = Image.fromarray(img_rgba)

    # Crop precisely to bounding box of the shield
    bbox = im_pil.getbbox()
    print(f"Shield bounding box: {bbox}")
    cropped = im_pil.crop(bbox)

    # Center in 1024x1024 square with 6.5% padding (height = 890px)
    target_h = 890
    cw, ch = cropped.size
    scale = target_h / ch
    target_w = int(cw * scale)
    scaled = cropped.resize((target_w, target_h), Image.Resampling.LANCZOS)

    canvas_1024 = Image.new('RGBA', (1024, 1024), (0, 0, 0, 0))
    offset_x = (1024 - target_w) // 2
    offset_y = (1024 - target_h) // 2
    canvas_1024.paste(scaled, (offset_x, offset_y), scaled)
    print(f"Created 1024x1024 transparent master (shield size {target_w}x{target_h})")

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
    src_logo = r"e:\Qaulium AI\Broswer\scratch\old_green_logo\old_qaulium_logo.png"
    master = create_transparent_master(src_logo)
    
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
