import os
import struct
from PIL import Image

def create_transparent_master(source_logo_path):
    print(f"Loading authoritative Quantum Shield logo from {source_logo_path}...")
    img = Image.open(source_logo_path).convert("RGBA")
    
    # Crop to the exact non-transparent bounding box of the shield
    bbox = img.getbbox()
    print(f"Shield content bounding box: {bbox}")
    cropped = img.crop(bbox)

    # Place centered into 1024x1024 canvas with slight padding (height = 920px for crisp margins)
    target_h = 920
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
    repo_root = r"e:\Qaulium AI\Broswer"
    src_logo = os.path.join(repo_root, "qualium", "chrome", "content", "qaulium_logo.png")
    master = create_transparent_master(src_logo)
    
    os.makedirs(os.path.join(repo_root, "scratch", "generated_icons"), exist_ok=True)
    master_1024_path = os.path.join(repo_root, "scratch", "generated_icons", "qualium_app_icon_1024.png")
    master.save(master_1024_path)
    print("Saved master 1024x1024 PNG.")

    # Save root master 1024 PNG
    master.save(os.path.join(repo_root, "qaulium_icon_1024.png"))

    # Save ICO with full multi-resolution suite
    ico_target = os.path.join(repo_root, "scratch", "generated_icons", "qualium.ico")
    save_ico_custom(master, ico_target, sizes=(256, 128, 64, 48, 32, 24, 16))

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
        out_p = os.path.join(repo_root, "scratch", "generated_icons", name)
        resized = master.resize((w, h), Image.Resampling.LANCZOS)
        resized.save(out_p, optimize=True)
        print(f"Generated {name} ({w}x{h})")

if __name__ == '__main__':
    main()
