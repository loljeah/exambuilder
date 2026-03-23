#!/usr/bin/env python3
"""
Pixel Art Sprite Generator for Knowledge Gate
Generates 64x64 sprites for creatures, hats, held items, and auras.
Uses the pixel-art-god skill guidelines:
- 8-16 colors per character
- Light from top-left
- Hue-shift shadows cool (blue/purple), highlights warm (yellow)
"""

from PIL import Image, ImageDraw
import os

# Cozy purple palette (matches GUI theme)
PALETTE = {
    # Base purples
    'bg': (30, 26, 46, 0),  # Transparent
    'dark_purple': (45, 35, 65),
    'mid_purple': (90, 70, 130),
    'light_purple': (140, 120, 180),

    # Cat colors
    'cat_dark': (60, 50, 70),
    'cat_mid': (100, 90, 110),
    'cat_light': (150, 140, 160),
    'cat_highlight': (200, 195, 210),
    'cat_nose': (255, 180, 180),

    # Slime colors (green-teal)
    'slime_dark': (30, 100, 90),
    'slime_mid': (60, 180, 150),
    'slime_light': (100, 220, 190),
    'slime_highlight': (180, 255, 230),
    'slime_shine': (255, 255, 255),

    # Octopus colors (pink-coral)
    'octo_dark': (130, 50, 80),
    'octo_mid': (200, 100, 130),
    'octo_light': (240, 150, 170),
    'octo_highlight': (255, 200, 210),

    # Snail colors (warm brown-cream)
    'snail_body_dark': (100, 80, 70),
    'snail_body_mid': (160, 130, 110),
    'snail_body_light': (200, 175, 155),
    'snail_shell_dark': (120, 60, 80),
    'snail_shell_mid': (180, 100, 120),
    'snail_shell_light': (220, 150, 160),

    # Eyes
    'eye_white': (255, 255, 255),
    'eye_dark': (30, 25, 40),
    'eye_shine': (255, 255, 255),

    # Mood indicators
    'happy_blush': (255, 180, 200),
    'sad_tear': (150, 200, 255),

    # Items
    'gold': (255, 215, 100),
    'gold_dark': (200, 160, 50),
    'silver': (200, 200, 210),
    'silver_dark': (150, 150, 160),
    'red': (220, 80, 80),
    'red_dark': (160, 50, 50),
    'blue': (100, 150, 220),
    'blue_dark': (60, 100, 160),
    'green': (100, 200, 120),
    'green_dark': (60, 140, 80),
    'white': (255, 255, 255),
    'black': (30, 25, 40),
}

def create_canvas():
    """Create a 64x64 transparent canvas."""
    return Image.new('RGBA', (64, 64), (0, 0, 0, 0))

def draw_pixel(draw, x, y, color, size=1):
    """Draw a pixel (or larger block) at coordinates."""
    if isinstance(color, str):
        color = PALETTE.get(color, (255, 0, 255))
    if len(color) == 3:
        color = (*color, 255)
    draw.rectangle([x, y, x + size - 1, y + size - 1], fill=color)

def draw_circle_filled(draw, cx, cy, r, color):
    """Draw a filled circle."""
    if isinstance(color, str):
        color = PALETTE.get(color, (255, 0, 255))
    if len(color) == 3:
        color = (*color, 255)
    draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=color)

def draw_ellipse_filled(draw, cx, cy, rx, ry, color):
    """Draw a filled ellipse."""
    if isinstance(color, str):
        color = PALETTE.get(color, (255, 0, 255))
    if len(color) == 3:
        color = (*color, 255)
    draw.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], fill=color)

# ============================================================
# CREATURE SPRITES
# ============================================================

def draw_cat(mood='neutral'):
    """Draw a cute pixel cat."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Body (rounded rectangle-ish)
    draw_ellipse_filled(draw, 32, 40, 18, 16, 'cat_mid')
    # Lighter belly
    draw_ellipse_filled(draw, 32, 44, 12, 10, 'cat_light')

    # Head
    draw_ellipse_filled(draw, 32, 22, 16, 14, 'cat_mid')
    # Face highlight
    draw_ellipse_filled(draw, 32, 24, 12, 10, 'cat_light')

    # Ears (triangular)
    for ex, flip in [(18, 1), (46, -1)]:
        for i in range(8):
            for j in range(8 - i):
                px = ex + (j * flip)
                py = 8 + i
                draw_pixel(draw, px, py, 'cat_mid')
        # Inner ear
        for i in range(4):
            for j in range(4 - i):
                px = ex + 2 * flip + (j * flip)
                py = 11 + i
                draw_pixel(draw, px, py, 'cat_nose')

    # Eyes
    eye_y = 20
    for ex in [24, 38]:
        draw_ellipse_filled(draw, ex, eye_y, 4, 5, 'eye_white')

        # Pupil position based on mood
        pupil_offset = 0
        if mood == 'happy':
            # Closed happy eyes (^_^)
            draw.line([ex - 3, eye_y, ex + 3, eye_y], fill=PALETTE['eye_dark'], width=2)
            continue
        elif mood == 'sad':
            pupil_offset = 1  # Looking down
        elif mood == 'lonely':
            pupil_offset = -1 if ex < 32 else 1  # Looking away

        draw_ellipse_filled(draw, ex + (1 if ex > 32 else -1), eye_y + pupil_offset, 2, 3, 'eye_dark')
        draw_pixel(draw, ex, eye_y - 2 + pupil_offset, 'eye_shine')

    # Nose
    draw_ellipse_filled(draw, 32, 28, 2, 2, 'cat_nose')

    # Mouth based on mood
    if mood == 'happy':
        # Big smile
        draw.arc([26, 26, 38, 36], 0, 180, fill=PALETTE['cat_dark'], width=2)
        # Blush
        draw_ellipse_filled(draw, 20, 26, 3, 2, 'happy_blush')
        draw_ellipse_filled(draw, 44, 26, 3, 2, 'happy_blush')
    elif mood == 'content':
        # Small smile
        draw.arc([28, 28, 36, 34], 0, 180, fill=PALETTE['cat_dark'], width=1)
    elif mood == 'sad':
        # Frown
        draw.arc([28, 30, 36, 38], 180, 360, fill=PALETTE['cat_dark'], width=1)
        # Tear
        draw_ellipse_filled(draw, 42, 26, 2, 3, 'sad_tear')
    elif mood == 'lonely':
        # Neutral line
        draw.line([28, 32, 36, 32], fill=PALETTE['cat_dark'], width=1)
    else:  # neutral
        draw.line([28, 31, 36, 31], fill=PALETTE['cat_dark'], width=1)

    # Tail
    for i in range(12):
        tx = 50 + i // 2
        ty = 36 + (i % 4) - 2
        draw_ellipse_filled(draw, tx, ty, 3, 3, 'cat_mid')

    # Paws
    for px in [22, 42]:
        draw_ellipse_filled(draw, px, 54, 5, 4, 'cat_light')

    # Whiskers
    for wy in [24, 28]:
        draw.line([14, wy, 22, wy + 1], fill=PALETTE['cat_dark'], width=1)
        draw.line([42, wy + 1, 50, wy], fill=PALETTE['cat_dark'], width=1)

    return img

def draw_slime(mood='neutral'):
    """Draw a cute bouncy slime."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Main body (blob shape)
    draw_ellipse_filled(draw, 32, 38, 22, 20, 'slime_mid')

    # Highlight/shine area (top-left lit)
    draw_ellipse_filled(draw, 26, 30, 12, 10, 'slime_light')

    # Big shine spot
    draw_ellipse_filled(draw, 20, 24, 4, 3, 'slime_highlight')
    draw_pixel(draw, 18, 22, 'slime_shine', 2)

    # Small shine
    draw_pixel(draw, 26, 20, 'slime_shine', 2)

    # Shadow underneath
    draw_ellipse_filled(draw, 32, 56, 18, 4, 'slime_dark')

    # Eyes
    eye_y = 34
    for ex in [24, 40]:
        if mood == 'happy':
            # Happy closed eyes (^_^)
            draw.arc([ex - 4, eye_y - 4, ex + 4, eye_y + 4], 200, 340, fill=PALETTE['eye_dark'], width=2)
        else:
            draw_ellipse_filled(draw, ex, eye_y, 4, 5, 'eye_white')

            pupil_offset = 0
            if mood == 'sad':
                pupil_offset = 2
            elif mood == 'lonely':
                pupil_offset = -2 if ex < 32 else 2

            draw_ellipse_filled(draw, ex, eye_y + pupil_offset, 2, 3, 'eye_dark')
            draw_pixel(draw, ex - 1, eye_y - 2 + pupil_offset, 'eye_shine')

    # Mouth
    if mood == 'happy':
        draw.arc([26, 38, 38, 48], 0, 180, fill=PALETTE['slime_dark'], width=2)
        # Blush
        draw_ellipse_filled(draw, 18, 40, 3, 2, 'happy_blush')
        draw_ellipse_filled(draw, 46, 40, 3, 2, 'happy_blush')
    elif mood == 'content':
        draw.arc([28, 40, 36, 46], 0, 180, fill=PALETTE['slime_dark'], width=1)
    elif mood == 'sad':
        draw.arc([28, 44, 36, 50], 180, 360, fill=PALETTE['slime_dark'], width=1)
        draw_ellipse_filled(draw, 44, 38, 2, 4, 'sad_tear')
    elif mood == 'lonely':
        draw.line([28, 44, 36, 44], fill=PALETTE['slime_dark'], width=1)
    else:
        draw.ellipse([30, 42, 34, 46], fill=PALETTE['slime_dark'])

    return img

def draw_octopus(mood='neutral'):
    """Draw a cute octopus."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Head (large dome)
    draw_ellipse_filled(draw, 32, 24, 20, 18, 'octo_mid')
    draw_ellipse_filled(draw, 28, 20, 12, 10, 'octo_light')

    # Tentacles (8 wavy appendages)
    tentacle_positions = [
        (12, 36), (18, 40), (24, 42), (30, 44),
        (34, 44), (40, 42), (46, 40), (52, 36)
    ]
    for i, (tx, ty) in enumerate(tentacle_positions):
        wave = (i % 2) * 2 - 1  # alternating wave direction
        for j in range(4):
            draw_ellipse_filled(draw, tx + wave * (j % 2), ty + j * 4, 4, 3, 'octo_mid')
            if j < 2:
                draw_ellipse_filled(draw, tx + wave * (j % 2), ty + j * 4, 3, 2, 'octo_light')

    # Eyes
    eye_y = 22
    for ex in [24, 40]:
        if mood == 'happy':
            draw.arc([ex - 4, eye_y - 4, ex + 4, eye_y + 4], 200, 340, fill=PALETTE['eye_dark'], width=2)
        else:
            draw_ellipse_filled(draw, ex, eye_y, 5, 6, 'eye_white')

            pupil_offset = 0
            if mood == 'sad':
                pupil_offset = 2
            elif mood == 'lonely':
                pupil_offset = -2 if ex < 32 else 2

            draw_ellipse_filled(draw, ex, eye_y + pupil_offset, 2, 3, 'eye_dark')
            draw_pixel(draw, ex - 1, eye_y - 2 + pupil_offset, 'eye_shine')

    # Mouth
    if mood == 'happy':
        draw.arc([26, 28, 38, 38], 0, 180, fill=PALETTE['octo_dark'], width=2)
        draw_ellipse_filled(draw, 18, 28, 3, 2, 'happy_blush')
        draw_ellipse_filled(draw, 46, 28, 3, 2, 'happy_blush')
    elif mood == 'content':
        draw.arc([28, 30, 36, 36], 0, 180, fill=PALETTE['octo_dark'], width=1)
    elif mood == 'sad':
        draw.arc([28, 32, 36, 38], 180, 360, fill=PALETTE['octo_dark'], width=1)
        draw_ellipse_filled(draw, 44, 26, 2, 4, 'sad_tear')
    elif mood == 'lonely':
        draw.line([28, 34, 36, 34], fill=PALETTE['octo_dark'], width=1)
    else:
        draw.ellipse([30, 32, 34, 36], fill=PALETTE['octo_dark'])

    # Spots on head
    for sx, sy in [(40, 14), (44, 20), (20, 16)]:
        draw_ellipse_filled(draw, sx, sy, 2, 2, 'octo_light')

    return img

def draw_snail(mood='neutral'):
    """Draw a cute snail."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Shell (spiral)
    draw_ellipse_filled(draw, 38, 30, 18, 16, 'snail_shell_mid')
    draw_ellipse_filled(draw, 40, 28, 14, 12, 'snail_shell_light')
    draw_ellipse_filled(draw, 42, 26, 8, 7, 'snail_shell_mid')
    draw_ellipse_filled(draw, 44, 24, 4, 3, 'snail_shell_dark')

    # Body
    draw_ellipse_filled(draw, 26, 46, 20, 10, 'snail_body_mid')
    draw_ellipse_filled(draw, 22, 44, 14, 7, 'snail_body_light')

    # Head
    draw_ellipse_filled(draw, 14, 38, 10, 12, 'snail_body_mid')
    draw_ellipse_filled(draw, 12, 36, 7, 8, 'snail_body_light')

    # Eye stalks
    for sx, stalk_x in [(10, -4), (18, 4)]:
        # Stalk
        draw.line([14 + stalk_x, 32, 14 + stalk_x, 20], fill=PALETTE['snail_body_mid'], width=3)
        # Eye
        if mood == 'happy':
            draw.arc([14 + stalk_x - 4, 16, 14 + stalk_x + 4, 24], 200, 340, fill=PALETTE['eye_dark'], width=2)
        else:
            draw_ellipse_filled(draw, 14 + stalk_x, 18, 4, 4, 'eye_white')

            pupil_offset = 0
            if mood == 'sad':
                pupil_offset = 1
            elif mood == 'lonely':
                pupil_offset = -1 if stalk_x < 0 else 1

            draw_ellipse_filled(draw, 14 + stalk_x, 18 + pupil_offset, 2, 2, 'eye_dark')
            draw_pixel(draw, 14 + stalk_x - 1, 16 + pupil_offset, 'eye_shine')

    # Mouth
    if mood == 'happy':
        draw.arc([8, 40, 18, 48], 0, 180, fill=PALETTE['snail_body_dark'], width=2)
        draw_ellipse_filled(draw, 6, 42, 2, 2, 'happy_blush')
        draw_ellipse_filled(draw, 22, 42, 2, 2, 'happy_blush')
    elif mood == 'content':
        draw.arc([10, 42, 18, 48], 0, 180, fill=PALETTE['snail_body_dark'], width=1)
    elif mood == 'sad':
        draw.arc([10, 44, 18, 50], 180, 360, fill=PALETTE['snail_body_dark'], width=1)
        draw_ellipse_filled(draw, 20, 36, 2, 3, 'sad_tear')
    elif mood == 'lonely':
        draw.line([10, 46, 18, 46], fill=PALETTE['snail_body_dark'], width=1)
    else:
        draw.ellipse([12, 44, 16, 48], fill=PALETTE['snail_body_dark'])

    # Slime trail
    for i in range(3):
        draw_ellipse_filled(draw, 50 + i * 6, 54, 3, 2, (180, 220, 200, 100))

    return img

# ============================================================
# HAT SPRITES
# ============================================================

def draw_wizard_hat():
    """Draw a wizard/witch hat."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Brim
    draw_ellipse_filled(draw, 32, 54, 26, 6, 'mid_purple')
    draw_ellipse_filled(draw, 28, 52, 18, 4, 'light_purple')

    # Cone
    for y in range(40):
        width = int(20 - (y * 0.45))
        if width > 0:
            draw.line([32 - width, 50 - y, 32 + width, 50 - y],
                     fill=PALETTE['mid_purple'] if y > 10 else PALETTE['dark_purple'])

    # Highlight on cone
    for y in range(30):
        width = int(8 - (y * 0.2))
        if width > 0:
            draw.line([24 - width, 48 - y, 24 + width, 48 - y], fill=PALETTE['light_purple'])

    # Star decoration
    draw_pixel(draw, 32, 16, 'gold', 3)
    draw_pixel(draw, 30, 18, 'gold', 2)
    draw_pixel(draw, 34, 18, 'gold', 2)

    return img

def draw_crown():
    """Draw a golden crown."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Base band
    draw.rectangle([12, 44, 52, 56], fill=PALETTE['gold'])
    draw.rectangle([14, 46, 50, 54], fill=PALETTE['gold_dark'])
    draw.rectangle([14, 44, 50, 48], fill=PALETTE['gold'])

    # Points
    for px in [16, 26, 36, 46]:
        for i in range(12):
            w = 4 - (i // 3)
            if w > 0:
                draw.line([px - w, 44 - i, px + w, 44 - i], fill=PALETTE['gold'])

    # Gems
    for px, color in [(21, 'red'), (32, 'blue'), (43, 'green')]:
        draw_ellipse_filled(draw, px, 50, 3, 3, color)
        draw_pixel(draw, px - 1, 48, 'white')

    return img

def draw_party_hat():
    """Draw a colorful party hat."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Cone with stripes
    colors = [PALETTE['red'], PALETTE['blue'], PALETTE['green'], PALETTE['gold']]
    for y in range(36):
        width = int(18 - (y * 0.45))
        if width > 0:
            color = colors[(y // 4) % len(colors)]
            draw.line([32 - width, 54 - y, 32 + width, 54 - y], fill=color)

    # Pom-pom on top
    draw_ellipse_filled(draw, 32, 16, 5, 5, 'gold')
    draw_pixel(draw, 30, 14, 'white', 2)

    # Elastic band suggestion
    draw.arc([20, 52, 44, 62], 0, 180, fill=PALETTE['dark_purple'], width=1)

    return img

def draw_top_hat():
    """Draw a fancy top hat."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Brim
    draw_ellipse_filled(draw, 32, 54, 24, 6, 'black')
    draw_ellipse_filled(draw, 28, 52, 16, 4, (60, 55, 70))

    # Cylinder
    draw.rectangle([16, 22, 48, 52], fill=PALETTE['black'])
    draw.rectangle([18, 24, 46, 50], fill=(50, 45, 60))

    # Top
    draw_ellipse_filled(draw, 32, 22, 16, 4, 'black')

    # Band
    draw.rectangle([16, 42, 48, 48], fill=PALETTE['red'])
    draw.rectangle([18, 44, 46, 46], fill=PALETTE['red_dark'])

    return img

def draw_cat_ears():
    """Draw cat ear headband."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Headband
    draw.arc([14, 44, 50, 60], 180, 360, fill=PALETTE['black'], width=3)

    # Ears
    for ex, flip in [(18, 1), (46, -1)]:
        # Outer ear
        for i in range(14):
            for j in range(14 - i):
                px = ex + (j * flip)
                py = 32 + i
                draw_pixel(draw, px, py, 'black')
        # Inner ear
        for i in range(8):
            for j in range(8 - i):
                px = ex + 3 * flip + (j * flip)
                py = 38 + i
                draw_pixel(draw, px, py, 'cat_nose')

    return img

def draw_halo():
    """Draw a glowing halo."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Outer glow
    draw_ellipse_filled(draw, 32, 20, 20, 6, (255, 240, 180, 80))
    # Ring
    draw.ellipse([14, 14, 50, 26], outline=PALETTE['gold'], width=4)
    # Inner highlight
    draw.arc([18, 16, 46, 24], 200, 340, fill=PALETTE['white'], width=2)

    return img

# ============================================================
# HELD ITEM SPRITES
# ============================================================

def draw_book():
    """Draw a study book."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Book cover
    draw.rectangle([18, 20, 46, 50], fill=PALETTE['red_dark'])
    draw.rectangle([20, 22, 44, 48], fill=PALETTE['red'])

    # Pages
    draw.rectangle([22, 24, 42, 46], fill=PALETTE['white'])
    # Page lines
    for y in range(28, 44, 3):
        draw.line([24, y, 40, y], fill=(200, 200, 210), width=1)

    # Spine
    draw.rectangle([18, 20, 22, 50], fill=PALETTE['red_dark'])

    # Bookmark
    draw.rectangle([36, 18, 40, 26], fill=PALETTE['gold'])

    return img

def draw_wand():
    """Draw a magic wand."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Wand stick
    draw.line([20, 50, 44, 20], fill=PALETTE['snail_body_dark'], width=4)
    draw.line([22, 48, 46, 18], fill=PALETTE['snail_body_mid'], width=2)

    # Star on top
    star_x, star_y = 46, 16
    draw_pixel(draw, star_x, star_y - 4, 'gold', 2)
    draw_pixel(draw, star_x - 4, star_y, 'gold', 2)
    draw_pixel(draw, star_x + 2, star_y, 'gold', 2)
    draw_pixel(draw, star_x - 2, star_y + 3, 'gold', 2)
    draw_pixel(draw, star_x + 1, star_y + 3, 'gold', 2)
    draw_pixel(draw, star_x, star_y, 'white', 2)

    # Sparkles
    for sx, sy in [(50, 12), (40, 8), (52, 20)]:
        draw_pixel(draw, sx, sy, 'white')

    return img

def draw_coffee():
    """Draw a coffee cup."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Cup body
    draw.rectangle([20, 28, 44, 54], fill=PALETTE['white'])
    draw.rectangle([22, 30, 42, 52], fill=(240, 235, 230))

    # Coffee
    draw_ellipse_filled(draw, 32, 32, 9, 4, 'snail_body_dark')

    # Handle
    draw.arc([40, 34, 52, 50], -60, 60, fill=PALETTE['white'], width=4)

    # Steam
    for i, sx in enumerate([28, 32, 36]):
        for j in range(3):
            sy = 22 - j * 4 - (i % 2) * 2
            draw_pixel(draw, sx + (j % 2) * 2, sy, (200, 200, 200, 150 - j * 40), 2)

    return img

def draw_pencil():
    """Draw a pencil."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Body (yellow)
    draw.polygon([(22, 48), (22, 24), (42, 24), (42, 48)], fill=PALETTE['gold'])
    draw.rectangle([24, 26, 40, 46], fill=(255, 220, 120))

    # Tip
    draw.polygon([(22, 48), (42, 48), (32, 58)], fill=PALETTE['snail_body_light'])
    draw.polygon([(28, 52), (36, 52), (32, 58)], fill=PALETTE['black'])

    # Eraser
    draw.rectangle([22, 18, 42, 24], fill=PALETTE['silver'])
    draw.rectangle([22, 14, 42, 18], fill=PALETTE['cat_nose'])

    # Metal band lines
    draw.line([22, 20, 42, 20], fill=PALETTE['silver_dark'], width=1)
    draw.line([22, 22, 42, 22], fill=PALETTE['silver_dark'], width=1)

    return img

def draw_trophy():
    """Draw a golden trophy."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Cup
    draw_ellipse_filled(draw, 32, 24, 14, 10, 'gold')
    draw_ellipse_filled(draw, 32, 22, 10, 6, 'gold_dark')
    draw_ellipse_filled(draw, 32, 20, 10, 6, 'gold')

    # Handles
    draw.arc([10, 18, 22, 32], 90, 270, fill=PALETTE['gold'], width=4)
    draw.arc([42, 18, 54, 32], 270, 90, fill=PALETTE['gold'], width=4)

    # Stem
    draw.rectangle([28, 32, 36, 44], fill=PALETTE['gold'])
    draw.rectangle([30, 34, 34, 42], fill=PALETTE['gold_dark'])

    # Base
    draw.rectangle([22, 44, 42, 48], fill=PALETTE['gold'])
    draw.rectangle([18, 48, 46, 54], fill=PALETTE['gold_dark'])
    draw.rectangle([20, 50, 44, 52], fill=PALETTE['gold'])

    # Star on cup
    draw_pixel(draw, 31, 22, 'white', 3)

    return img

def draw_controller():
    """Draw a game controller."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Body
    draw.rounded_rectangle([12, 24, 52, 48], radius=8, fill=PALETTE['dark_purple'])
    draw.rounded_rectangle([14, 26, 50, 46], radius=6, fill=PALETTE['mid_purple'])

    # D-pad
    draw.rectangle([18, 32, 28, 36], fill=PALETTE['black'])
    draw.rectangle([21, 29, 25, 39], fill=PALETTE['black'])

    # Buttons
    for bx, by, color in [(40, 30, 'red'), (46, 34, 'blue'), (40, 38, 'green'), (34, 34, 'gold')]:
        draw_ellipse_filled(draw, bx, by, 3, 3, color)

    # Analog sticks
    draw_ellipse_filled(draw, 26, 42, 4, 4, 'black')
    draw_ellipse_filled(draw, 38, 42, 4, 4, 'black')

    return img

# ============================================================
# AURA SPRITES (transparent overlays)
# ============================================================

def draw_sparkle_aura():
    """Draw sparkle effect overlay."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    sparkle_positions = [
        (8, 12), (56, 8), (4, 40), (60, 44),
        (12, 56), (52, 58), (32, 4), (48, 24),
        (16, 28), (44, 52)
    ]

    for x, y in sparkle_positions:
        # Four-point sparkle
        size = 2 + (x + y) % 2
        color = (255, 255, 200, 180)
        draw_pixel(draw, x, y, color, 2)
        draw_pixel(draw, x - size, y, color)
        draw_pixel(draw, x + size, y, color)
        draw_pixel(draw, x, y - size, color)
        draw_pixel(draw, x, y + size, color)

    return img

def draw_hearts_aura():
    """Draw floating hearts overlay."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    heart_positions = [(10, 10), (54, 16), (8, 48), (56, 52), (30, 6), (48, 38)]

    for hx, hy in heart_positions:
        # Simple heart shape
        size = 3 + (hx % 2)
        color = (255, 150, 180, 200)
        draw_ellipse_filled(draw, hx - size//2, hy, size, size, color)
        draw_ellipse_filled(draw, hx + size//2, hy, size, size, color)
        # Bottom point
        for i in range(size + 1):
            w = size - i
            draw.line([hx - w, hy + i + size//2, hx + w, hy + i + size//2], fill=color)

    return img

def draw_stars_aura():
    """Draw star field overlay."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    star_positions = [
        (6, 8), (58, 12), (12, 52), (50, 56),
        (28, 4), (36, 60), (4, 30), (60, 34)
    ]

    for sx, sy in star_positions:
        size = 2 + (sx + sy) % 2
        color = (255, 255, 150, 200)
        # Simple 4-point star
        draw_pixel(draw, sx, sy, color, 2)
        for i in range(1, size + 1):
            alpha = 200 - i * 40
            c = (255, 255, 150, max(alpha, 50))
            draw_pixel(draw, sx - i, sy, c)
            draw_pixel(draw, sx + i, sy, c)
            draw_pixel(draw, sx, sy - i, c)
            draw_pixel(draw, sx, sy + i, c)

    return img

def draw_flame_aura():
    """Draw flame effect overlay."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    flame_bases = [(8, 58), (20, 60), (32, 62), (44, 60), (56, 58)]

    for fx, fy in flame_bases:
        # Flame colors from base to tip
        colors = [
            (255, 100, 50, 180),   # Orange
            (255, 150, 50, 160),   # Yellow-orange
            (255, 200, 100, 140),  # Yellow
            (255, 255, 200, 100),  # Pale yellow
        ]

        height = 12 + (fx % 4) * 2
        for i in range(height):
            width = max(1, 4 - i // 3)
            color_idx = min(i // 3, len(colors) - 1)
            for w in range(-width, width + 1):
                y = fy - i
                x = fx + w + ((i % 3) - 1)  # Wavey effect
                if 0 <= x < 64 and 0 <= y < 64:
                    draw_pixel(draw, x, y, colors[color_idx])

    return img

def draw_rainbow_aura():
    """Draw rainbow ring overlay."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Rainbow colors
    colors = [
        (255, 100, 100, 100),  # Red
        (255, 180, 100, 100),  # Orange
        (255, 255, 100, 100),  # Yellow
        (100, 255, 100, 100),  # Green
        (100, 200, 255, 100),  # Blue
        (150, 100, 255, 100),  # Indigo
        (200, 100, 255, 100),  # Violet
    ]

    # Draw concentric arcs
    for i, color in enumerate(colors):
        r = 28 - i * 2
        if r > 0:
            draw.arc([32 - r, 32 - r, 32 + r, 32 + r], 0, 360, fill=color, width=2)

    return img

# ============================================================
# BACKGROUND SPRITES
# ============================================================

def draw_starfield_bg():
    """Draw a starry night background."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Dark purple gradient base
    for y in range(64):
        color = (20 + y // 4, 15 + y // 5, 35 + y // 3, 255)
        draw.line([0, y, 63, y], fill=color)

    # Stars
    import random
    random.seed(42)  # Reproducible
    for _ in range(30):
        x, y = random.randint(0, 63), random.randint(0, 50)
        brightness = random.randint(150, 255)
        size = 1 if random.random() > 0.3 else 2
        draw_pixel(draw, x, y, (brightness, brightness, brightness - 20, 255), size)

    return img

def draw_cozy_room_bg():
    """Draw a cozy room background."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Wall
    draw.rectangle([0, 0, 63, 40], fill=(60, 50, 80))

    # Floor
    draw.rectangle([0, 40, 63, 63], fill=(80, 60, 50))

    # Floorboard lines
    for x in range(0, 64, 12):
        draw.line([x, 40, x, 63], fill=(60, 45, 35), width=1)

    # Window
    draw.rectangle([20, 8, 44, 30], fill=(100, 120, 150))
    draw.rectangle([22, 10, 42, 28], fill=(150, 180, 220))
    draw.line([32, 10, 32, 28], fill=(80, 70, 90), width=2)
    draw.line([22, 19, 42, 19], fill=(80, 70, 90), width=2)

    # Picture frame on wall
    draw.rectangle([50, 12, 60, 24], fill=(100, 70, 50))
    draw.rectangle([52, 14, 58, 22], fill=(180, 150, 200))

    return img

def draw_garden_bg():
    """Draw a garden background."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Sky
    for y in range(40):
        blue = 150 + (40 - y)
        draw.line([0, y, 63, y], fill=(120, 180, min(255, blue)))

    # Grass
    draw.rectangle([0, 40, 63, 63], fill=(80, 160, 90))

    # Grass texture
    import random
    random.seed(123)
    for _ in range(40):
        x = random.randint(0, 63)
        h = random.randint(3, 8)
        draw.line([x, 40, x, 40 - h], fill=(60, 140, 70), width=1)

    # Flowers
    for fx, fy, color in [(10, 48, 'red'), (25, 52, 'gold'), (45, 50, 'blue'), (55, 54, 'cat_nose')]:
        draw_ellipse_filled(draw, fx, fy, 3, 3, color)
        draw.line([fx, fy + 3, fx, fy + 8], fill=(50, 120, 60), width=2)

    # Sun
    draw_ellipse_filled(draw, 54, 10, 8, 8, 'gold')

    return img

def draw_library_bg():
    """Draw a library/study background."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Wall
    draw.rectangle([0, 0, 63, 63], fill=(50, 40, 60))

    # Bookshelf
    draw.rectangle([4, 8, 60, 56], fill=(80, 50, 40))

    # Shelves
    for y in [20, 36, 52]:
        draw.rectangle([6, y, 58, y + 2], fill=(60, 35, 25))

    # Books on shelves
    colors = ['red', 'blue', 'green', 'gold', 'mid_purple']
    import random
    random.seed(456)
    for shelf_y in [10, 26, 42]:
        x = 8
        while x < 54:
            w = random.randint(4, 8)
            h = random.randint(8, 12)
            color = random.choice(colors)
            draw.rectangle([x, shelf_y + (14 - h), x + w, shelf_y + 14], fill=PALETTE.get(color, PALETTE['mid_purple']))
            x += w + 1

    return img

def draw_clouds_bg():
    """Draw a dreamy clouds background."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    # Sky gradient
    for y in range(64):
        purple = 120 + y
        blue = 150 + y // 2
        draw.line([0, y, 63, y], fill=(min(255, purple), 140, min(255, blue)))

    # Clouds
    cloud_positions = [(10, 15), (35, 25), (55, 12), (20, 40), (45, 45)]
    for cx, cy in cloud_positions:
        for dx, dy, r in [(0, 0, 8), (-6, 2, 5), (6, 2, 5), (-3, -3, 4), (3, -3, 4)]:
            draw_ellipse_filled(draw, cx + dx, cy + dy, r, r // 2 + 2, (255, 255, 255, 180))

    return img

# ============================================================
# MAIN GENERATION
# ============================================================

def ensure_dir(path):
    """Create directory if it doesn't exist."""
    os.makedirs(path, exist_ok=True)

def main():
    base_dir = os.path.dirname(os.path.abspath(__file__))

    moods = ['happy', 'content', 'neutral', 'sad', 'lonely']
    creatures = {
        'cat': draw_cat,
        'slime': draw_slime,
        'octopus': draw_octopus,
        'snail': draw_snail,
    }

    # Generate creature sprites
    for creature_name, draw_func in creatures.items():
        creature_dir = os.path.join(base_dir, creature_name)
        ensure_dir(creature_dir)

        for mood in moods:
            img = draw_func(mood)
            filepath = os.path.join(creature_dir, f'{mood}.png')
            img.save(filepath)
            print(f'Generated: {filepath}')

    # Generate hat sprites
    hats_dir = os.path.join(base_dir, 'hats')
    ensure_dir(hats_dir)

    hat_funcs = {
        'wizard': draw_wizard_hat,
        'crown': draw_crown,
        'party': draw_party_hat,
        'tophat': draw_top_hat,
        'catears': draw_cat_ears,
        'halo': draw_halo,
    }

    for hat_name, draw_func in hat_funcs.items():
        img = draw_func()
        filepath = os.path.join(hats_dir, f'{hat_name}.png')
        img.save(filepath)
        print(f'Generated: {filepath}')

    # Generate held item sprites
    held_dir = os.path.join(base_dir, 'held')
    ensure_dir(held_dir)

    held_funcs = {
        'book': draw_book,
        'wand': draw_wand,
        'coffee': draw_coffee,
        'pencil': draw_pencil,
        'trophy': draw_trophy,
        'controller': draw_controller,
    }

    for item_name, draw_func in held_funcs.items():
        img = draw_func()
        filepath = os.path.join(held_dir, f'{item_name}.png')
        img.save(filepath)
        print(f'Generated: {filepath}')

    # Generate aura sprites
    auras_dir = os.path.join(base_dir, 'auras')
    ensure_dir(auras_dir)

    aura_funcs = {
        'sparkles': draw_sparkle_aura,
        'hearts': draw_hearts_aura,
        'stars': draw_stars_aura,
        'flames': draw_flame_aura,
        'rainbow': draw_rainbow_aura,
    }

    for aura_name, draw_func in aura_funcs.items():
        img = draw_func()
        filepath = os.path.join(auras_dir, f'{aura_name}.png')
        img.save(filepath)
        print(f'Generated: {filepath}')

    # Generate background sprites
    backgrounds_dir = os.path.join(base_dir, 'backgrounds')
    ensure_dir(backgrounds_dir)

    bg_funcs = {
        'starfield': draw_starfield_bg,
        'cozy_room': draw_cozy_room_bg,
        'garden': draw_garden_bg,
        'library': draw_library_bg,
        'clouds': draw_clouds_bg,
    }

    for bg_name, draw_func in bg_funcs.items():
        img = draw_func()
        filepath = os.path.join(backgrounds_dir, f'{bg_name}.png')
        img.save(filepath)
        print(f'Generated: {filepath}')

    print('\nAll sprites generated successfully!')
    print(f'Total: {len(creatures) * len(moods)} creature sprites')
    print(f'       {len(hat_funcs)} hat sprites')
    print(f'       {len(held_funcs)} held item sprites')
    print(f'       {len(aura_funcs)} aura sprites')
    print(f'       {len(bg_funcs)} background sprites')

if __name__ == '__main__':
    main()
