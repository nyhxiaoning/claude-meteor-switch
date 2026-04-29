#!/usr/bin/env python3
from PIL import Image, ImageDraw, ImageFont
import os

def create_icon(size, filename):
    # 创建透明背景
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # 黑色主题
    primary_color = (0, 0, 0, 255)
    
    # 绘制圆角背景
    corner_radius = size // 6
    padding = size // 10
    draw.rounded_rectangle(
        [(padding, padding), (size - padding, size - padding)],
        radius=corner_radius,
        fill=primary_color
    )
    
    # 绘制 "M" 字母
    try:
        # 尝试使用系统字体
        font_size = size // 2
        try:
            font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', font_size)
        except:
            font = ImageFont.truetype('/System/Library/Fonts/Arial.ttf', font_size)
    except:
        # 如果找不到字体，使用简单的几何图形
        center_x = size // 2
        center_y = size // 2
        line_width = size // 10
        
        # 简单的 M 字形
        margin = size // 3
        draw.line([(margin, margin), (margin, size - margin)], fill=(255, 255, 255, 255), width=line_width)
        draw.line([(size - margin, margin), (size - margin, size - margin)], fill=(255, 255, 255, 255), width=line_width)
        draw.line([(margin, margin), (center_x, center_y)], fill=(255, 255, 255, 255), width=line_width)
        draw.line([(size - margin, margin), (center_x, center_y)], fill=(255, 255, 255, 255), width=line_width)
    else:
        # 使用字体
        text = "M"
        bbox = draw.textbbox((0, 0), text, font=font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        x = (size - text_width) // 2 - bbox[0]
        y = (size - text_height) // 2 - bbox[1]
        draw.text((x, y), text, fill=(255, 255, 255, 255), font=font)
    
    img.save(filename)
    print(f"Created {filename}")

def main():
    sizes = [32, 64, 128, 256, 512]
    icons_dir = 'src-tauri/icons'
    
    for size in sizes:
        if size == 32:
            create_icon(size, f'{icons_dir}/32x32.png')
        elif size == 64:
            create_icon(size, f'{icons_dir}/64x64.png')
        elif size == 128:
            create_icon(size, f'{icons_dir}/128x128.png')
            create_icon(size, f'{icons_dir}/128x128@2x.png')
        elif size == 512:
            create_icon(size, f'{icons_dir}/icon.png')
    
    print(f"All icons generated successfully in {icons_dir}/")
    print("\nNext steps:")
    print("1. Run 'pnpm tauri build' to rebuild the app with new icons")
    print("2. Or for dev mode, you'll see the new icon after restart")

if __name__ == '__main__':
    main()
