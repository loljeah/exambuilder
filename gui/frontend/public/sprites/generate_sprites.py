#!/usr/bin/env python3
"""
Pixel Art Sprite Generator for Knowledge Gate
Generates 64x64 sprites for creatures, hats, held items, auras, and backgrounds.
Uses the pixel-art-god skill guidelines:
- 8-16 colors per character
- Light from top-left
- Hue-shift shadows cool (blue/purple), highlights warm (yellow)
"""

from PIL import Image, ImageDraw
import os
import random

# Cozy purple palette (matches GUI theme)
PALETTE = {
    # Base purples
    'bg': (30, 26, 46, 0),  # Transparent
    'dark_purple': (45, 35, 65),
    'mid_purple': (90, 70, 130),
    'light_purple': (140, 120, 180),
    'pale_purple': (180, 160, 210),

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

    # Items / colors
    'gold': (255, 215, 100),
    'gold_dark': (200, 160, 50),
    'gold_light': (255, 240, 180),
    'silver': (200, 200, 210),
    'silver_dark': (150, 150, 160),
    'silver_light': (230, 230, 240),
    'bronze': (180, 120, 80),
    'bronze_dark': (140, 90, 60),
    'red': (220, 80, 80),
    'red_dark': (160, 50, 50),
    'red_light': (255, 140, 140),
    'blue': (100, 150, 220),
    'blue_dark': (60, 100, 160),
    'blue_light': (150, 200, 255),
    'green': (100, 200, 120),
    'green_dark': (60, 140, 80),
    'green_light': (150, 230, 170),
    'yellow': (255, 230, 100),
    'yellow_dark': (200, 180, 60),
    'orange': (255, 160, 80),
    'orange_dark': (200, 120, 50),
    'pink': (255, 150, 200),
    'pink_dark': (200, 100, 150),
    'cyan': (100, 230, 230),
    'cyan_dark': (60, 180, 180),
    'teal': (80, 180, 170),
    'teal_dark': (50, 130, 120),
    'white': (255, 255, 255),
    'black': (30, 25, 40),
    'brown': (120, 80, 60),
    'brown_dark': (80, 50, 40),
    'brown_light': (160, 120, 90),
    'gray': (120, 120, 130),
    'gray_dark': (80, 80, 90),
    'gray_light': (180, 180, 190),
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
        if mood == 'happy':
            draw.line([ex - 3, eye_y, ex + 3, eye_y], fill=PALETTE['eye_dark'], width=2)
            continue
        pupil_offset = 1 if mood == 'sad' else (-1 if mood == 'lonely' and ex < 32 else (1 if mood == 'lonely' else 0))
        draw_ellipse_filled(draw, ex + (1 if ex > 32 else -1), eye_y + pupil_offset, 2, 3, 'eye_dark')
        draw_pixel(draw, ex, eye_y - 2 + pupil_offset, 'eye_shine')

    # Nose
    draw_ellipse_filled(draw, 32, 28, 2, 2, 'cat_nose')

    # Mouth based on mood
    if mood == 'happy':
        draw.arc([26, 26, 38, 36], 0, 180, fill=PALETTE['cat_dark'], width=2)
        draw_ellipse_filled(draw, 20, 26, 3, 2, 'happy_blush')
        draw_ellipse_filled(draw, 44, 26, 3, 2, 'happy_blush')
    elif mood == 'content':
        draw.arc([28, 28, 36, 34], 0, 180, fill=PALETTE['cat_dark'], width=1)
    elif mood == 'sad':
        draw.arc([28, 30, 36, 38], 180, 360, fill=PALETTE['cat_dark'], width=1)
        draw_ellipse_filled(draw, 42, 26, 2, 3, 'sad_tear')
    elif mood == 'lonely':
        draw.line([28, 32, 36, 32], fill=PALETTE['cat_dark'], width=1)
    else:
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

    draw_ellipse_filled(draw, 32, 38, 22, 20, 'slime_mid')
    draw_ellipse_filled(draw, 26, 30, 12, 10, 'slime_light')
    draw_ellipse_filled(draw, 20, 24, 4, 3, 'slime_highlight')
    draw_pixel(draw, 18, 22, 'slime_shine', 2)
    draw_pixel(draw, 26, 20, 'slime_shine', 2)
    draw_ellipse_filled(draw, 32, 56, 18, 4, 'slime_dark')

    eye_y = 34
    for ex in [24, 40]:
        if mood == 'happy':
            draw.arc([ex - 4, eye_y - 4, ex + 4, eye_y + 4], 200, 340, fill=PALETTE['eye_dark'], width=2)
        else:
            draw_ellipse_filled(draw, ex, eye_y, 4, 5, 'eye_white')
            pupil_offset = 2 if mood == 'sad' else (-2 if mood == 'lonely' and ex < 32 else (2 if mood == 'lonely' else 0))
            draw_ellipse_filled(draw, ex, eye_y + pupil_offset, 2, 3, 'eye_dark')
            draw_pixel(draw, ex - 1, eye_y - 2 + pupil_offset, 'eye_shine')

    if mood == 'happy':
        draw.arc([26, 38, 38, 48], 0, 180, fill=PALETTE['slime_dark'], width=2)
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

    draw_ellipse_filled(draw, 32, 24, 20, 18, 'octo_mid')
    draw_ellipse_filled(draw, 28, 20, 12, 10, 'octo_light')

    tentacle_positions = [(12, 36), (18, 40), (24, 42), (30, 44), (34, 44), (40, 42), (46, 40), (52, 36)]
    for i, (tx, ty) in enumerate(tentacle_positions):
        wave = (i % 2) * 2 - 1
        for j in range(4):
            draw_ellipse_filled(draw, tx + wave * (j % 2), ty + j * 4, 4, 3, 'octo_mid')
            if j < 2:
                draw_ellipse_filled(draw, tx + wave * (j % 2), ty + j * 4, 3, 2, 'octo_light')

    eye_y = 22
    for ex in [24, 40]:
        if mood == 'happy':
            draw.arc([ex - 4, eye_y - 4, ex + 4, eye_y + 4], 200, 340, fill=PALETTE['eye_dark'], width=2)
        else:
            draw_ellipse_filled(draw, ex, eye_y, 5, 6, 'eye_white')
            pupil_offset = 2 if mood == 'sad' else (-2 if mood == 'lonely' and ex < 32 else (2 if mood == 'lonely' else 0))
            draw_ellipse_filled(draw, ex, eye_y + pupil_offset, 2, 3, 'eye_dark')
            draw_pixel(draw, ex - 1, eye_y - 2 + pupil_offset, 'eye_shine')

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

    for sx, sy in [(40, 14), (44, 20), (20, 16)]:
        draw_ellipse_filled(draw, sx, sy, 2, 2, 'octo_light')

    return img

def draw_snail(mood='neutral'):
    """Draw a cute snail."""
    img = create_canvas()
    draw = ImageDraw.Draw(img)

    draw_ellipse_filled(draw, 38, 30, 18, 16, 'snail_shell_mid')
    draw_ellipse_filled(draw, 40, 28, 14, 12, 'snail_shell_light')
    draw_ellipse_filled(draw, 42, 26, 8, 7, 'snail_shell_mid')
    draw_ellipse_filled(draw, 44, 24, 4, 3, 'snail_shell_dark')

    draw_ellipse_filled(draw, 26, 46, 20, 10, 'snail_body_mid')
    draw_ellipse_filled(draw, 22, 44, 14, 7, 'snail_body_light')
    draw_ellipse_filled(draw, 14, 38, 10, 12, 'snail_body_mid')
    draw_ellipse_filled(draw, 12, 36, 7, 8, 'snail_body_light')

    for sx, stalk_x in [(10, -4), (18, 4)]:
        draw.line([14 + stalk_x, 32, 14 + stalk_x, 20], fill=PALETTE['snail_body_mid'], width=3)
        if mood == 'happy':
            draw.arc([14 + stalk_x - 4, 16, 14 + stalk_x + 4, 24], 200, 340, fill=PALETTE['eye_dark'], width=2)
        else:
            draw_ellipse_filled(draw, 14 + stalk_x, 18, 4, 4, 'eye_white')
            pupil_offset = 1 if mood == 'sad' else (-1 if mood == 'lonely' and stalk_x < 0 else (1 if mood == 'lonely' else 0))
            draw_ellipse_filled(draw, 14 + stalk_x, 18 + pupil_offset, 2, 2, 'eye_dark')
            draw_pixel(draw, 14 + stalk_x - 1, 16 + pupil_offset, 'eye_shine')

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

    for i in range(3):
        draw_ellipse_filled(draw, 50 + i * 6, 54, 3, 2, (180, 220, 200, 100))

    return img

# ============================================================
# HAT SPRITES (30 hats)
# ============================================================

HAT_GENERATORS = {}

def hat(name):
    def decorator(func):
        HAT_GENERATORS[name] = func
        return func
    return decorator

@hat('wizard')
def draw_wizard_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 54, 26, 6, 'mid_purple')
    draw_ellipse_filled(draw, 28, 52, 18, 4, 'light_purple')
    for y in range(40):
        width = int(20 - (y * 0.45))
        if width > 0:
            draw.line([32 - width, 50 - y, 32 + width, 50 - y], fill=PALETTE['mid_purple'] if y > 10 else PALETTE['dark_purple'])
    for y in range(30):
        width = int(8 - (y * 0.2))
        if width > 0:
            draw.line([24 - width, 48 - y, 24 + width, 48 - y], fill=PALETTE['light_purple'])
    draw_pixel(draw, 32, 16, 'gold', 3)
    draw_pixel(draw, 30, 18, 'gold', 2)
    draw_pixel(draw, 34, 18, 'gold', 2)
    return img

@hat('crown')
def draw_crown():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([12, 44, 52, 56], fill=PALETTE['gold'])
    draw.rectangle([14, 46, 50, 54], fill=PALETTE['gold_dark'])
    draw.rectangle([14, 44, 50, 48], fill=PALETTE['gold'])
    for px in [16, 26, 36, 46]:
        for i in range(12):
            w = 4 - (i // 3)
            if w > 0:
                draw.line([px - w, 44 - i, px + w, 44 - i], fill=PALETTE['gold'])
    for px, color in [(21, 'red'), (32, 'blue'), (43, 'green')]:
        draw_ellipse_filled(draw, px, 50, 3, 3, color)
        draw_pixel(draw, px - 1, 48, 'white')
    return img

@hat('party')
def draw_party_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    colors = [PALETTE['red'], PALETTE['blue'], PALETTE['green'], PALETTE['gold']]
    for y in range(36):
        width = int(18 - (y * 0.45))
        if width > 0:
            color = colors[(y // 4) % len(colors)]
            draw.line([32 - width, 54 - y, 32 + width, 54 - y], fill=color)
    draw_ellipse_filled(draw, 32, 16, 5, 5, 'gold')
    draw_pixel(draw, 30, 14, 'white', 2)
    draw.arc([20, 52, 44, 62], 0, 180, fill=PALETTE['dark_purple'], width=1)
    return img

@hat('tophat')
def draw_top_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 54, 24, 6, 'black')
    draw_ellipse_filled(draw, 28, 52, 16, 4, (60, 55, 70))
    draw.rectangle([16, 22, 48, 52], fill=PALETTE['black'])
    draw.rectangle([18, 24, 46, 50], fill=(50, 45, 60))
    draw_ellipse_filled(draw, 32, 22, 16, 4, 'black')
    draw.rectangle([16, 42, 48, 48], fill=PALETTE['red'])
    draw.rectangle([18, 44, 46, 46], fill=PALETTE['red_dark'])
    return img

@hat('catears')
def draw_cat_ears():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.arc([14, 44, 50, 60], 180, 360, fill=PALETTE['black'], width=3)
    for ex, flip in [(18, 1), (46, -1)]:
        for i in range(14):
            for j in range(14 - i):
                draw_pixel(draw, ex + (j * flip), 32 + i, 'black')
        for i in range(8):
            for j in range(8 - i):
                draw_pixel(draw, ex + 3 * flip + (j * flip), 38 + i, 'cat_nose')
    return img

@hat('halo')
def draw_halo():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 20, 20, 6, (255, 240, 180, 80))
    draw.ellipse([14, 14, 50, 26], outline=PALETTE['gold'], width=4)
    draw.arc([18, 16, 46, 24], 200, 340, fill=PALETTE['white'], width=2)
    return img

@hat('beret')
def draw_beret():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 46, 22, 10, 'red')
    draw_ellipse_filled(draw, 28, 42, 16, 8, 'red_light')
    draw_ellipse_filled(draw, 24, 38, 6, 6, 'red')
    draw_ellipse_filled(draw, 28, 42, 3, 3, 'black')
    return img

@hat('beanie')
def draw_beanie():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 44, 20, 12, 'blue')
    draw_ellipse_filled(draw, 32, 40, 16, 8, 'blue_light')
    draw.rectangle([14, 48, 50, 56], fill=PALETTE['blue_dark'])
    for x in range(14, 50, 4):
        draw.line([x, 48, x, 56], fill=PALETTE['blue'], width=2)
    draw_ellipse_filled(draw, 32, 26, 6, 6, 'blue')
    return img

@hat('cowboy')
def draw_cowboy_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 52, 28, 8, 'brown')
    draw_ellipse_filled(draw, 32, 50, 24, 5, 'brown_light')
    draw_ellipse_filled(draw, 32, 38, 16, 12, 'brown')
    draw_ellipse_filled(draw, 32, 36, 12, 8, 'brown_light')
    draw.rectangle([20, 44, 44, 48], fill=PALETTE['gold_dark'])
    return img

@hat('chef')
def draw_chef_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 30, 18, 20, 'white')
    draw_ellipse_filled(draw, 24, 28, 8, 8, 'white')
    draw_ellipse_filled(draw, 40, 28, 8, 8, 'white')
    draw_ellipse_filled(draw, 32, 22, 10, 10, 'white')
    draw.rectangle([16, 46, 48, 54], fill=PALETTE['white'])
    draw.rectangle([18, 48, 46, 52], fill=PALETTE['gray_light'])
    return img

@hat('pirate')
def draw_pirate_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 42, 24, 14, 'black')
    for i in range(16):
        w = 22 - i
        if w > 0:
            draw.line([32 - w, 40 - i, 32 + w, 40 - i], fill=PALETTE['black'])
    draw_ellipse_filled(draw, 32, 38, 8, 8, 'white')
    draw.line([28, 34, 36, 42], fill=PALETTE['black'], width=2)
    draw.line([28, 42, 36, 34], fill=PALETTE['black'], width=2)
    return img

@hat('viking')
def draw_viking_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 44, 20, 12, 'brown')
    draw_ellipse_filled(draw, 32, 42, 16, 8, 'brown_light')
    for side in [-1, 1]:
        for i in range(16):
            x = 32 + side * (20 + i // 2)
            y = 38 - i + abs(i - 8)
            draw_ellipse_filled(draw, x, y, 4, 3, 'white')
    return img

@hat('propeller')
def draw_propeller_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 46, 18, 10, 'blue')
    draw_ellipse_filled(draw, 32, 44, 14, 7, 'blue_light')
    draw_ellipse_filled(draw, 32, 32, 3, 3, 'red')
    for angle in [0, 120, 240]:
        import math
        rad = math.radians(angle)
        for i in range(12):
            x = 32 + int(math.cos(rad) * i)
            y = 32 + int(math.sin(rad) * i)
            draw_pixel(draw, x, y, 'yellow', 2)
    return img

@hat('bunny_ears')
def draw_bunny_ears():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.arc([14, 50, 50, 60], 180, 360, fill=PALETTE['pink'], width=3)
    for ex in [20, 44]:
        draw_ellipse_filled(draw, ex, 28, 6, 20, 'pink')
        draw_ellipse_filled(draw, ex, 30, 3, 14, 'pink_dark')
    return img

@hat('santa')
def draw_santa_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(30):
        w = int(20 - y * 0.5)
        if w > 0:
            draw.line([32 - w + y//3, 52 - y, 32 + w + y//3, 52 - y], fill=PALETTE['red'])
    draw.rectangle([12, 50, 52, 58], fill=PALETTE['white'])
    draw_ellipse_filled(draw, 48, 22, 6, 6, 'white')
    return img

@hat('graduation')
def draw_graduation_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.polygon([(8, 42), (32, 30), (56, 42), (32, 54)], fill=PALETTE['black'])
    draw.polygon([(12, 42), (32, 32), (52, 42), (32, 52)], fill=(50, 50, 60))
    draw_ellipse_filled(draw, 32, 42, 6, 4, 'black')
    draw.line([32, 42, 48, 52], fill=PALETTE['gold'], width=2)
    draw_ellipse_filled(draw, 48, 54, 4, 4, 'gold')
    return img

@hat('fez')
def draw_fez():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 50, 16, 8, 'red')
    for y in range(20):
        w = int(14 - y * 0.3)
        if w > 0:
            draw.line([32 - w, 48 - y, 32 + w, 48 - y], fill=PALETTE['red'])
    draw_ellipse_filled(draw, 32, 28, 8, 4, 'red_dark')
    draw.line([32, 28, 44, 40], fill=PALETTE['gold'], width=2)
    draw_ellipse_filled(draw, 44, 42, 4, 4, 'gold')
    return img

@hat('headphones')
def draw_headphones():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.arc([14, 20, 50, 50], 180, 360, fill=PALETTE['gray_dark'], width=4)
    for x in [14, 50]:
        draw_ellipse_filled(draw, x, 46, 8, 10, 'gray_dark')
        draw_ellipse_filled(draw, x, 46, 5, 7, 'gray')
    return img

@hat('flower_crown')
def draw_flower_crown():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.arc([14, 40, 50, 56], 180, 360, fill=PALETTE['green'], width=3)
    colors = ['pink', 'yellow', 'red', 'blue', 'pink']
    for i, x in enumerate([16, 24, 32, 40, 48]):
        draw_ellipse_filled(draw, x, 44, 5, 5, colors[i])
        draw_ellipse_filled(draw, x, 44, 2, 2, 'yellow')
    return img

@hat('tiara')
def draw_tiara():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.arc([14, 44, 50, 58], 180, 360, fill=PALETTE['silver'], width=3)
    for x, h in [(24, 8), (32, 14), (40, 8)]:
        for i in range(h):
            draw_pixel(draw, x - 1, 48 - i, 'silver')
            draw_pixel(draw, x, 48 - i, 'silver')
            draw_pixel(draw, x + 1, 48 - i, 'silver')
    draw_ellipse_filled(draw, 32, 34, 4, 4, 'cyan')
    draw_pixel(draw, 31, 33, 'white')
    return img

@hat('baseball')
def draw_baseball_cap():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 44, 18, 10, 'red')
    draw_ellipse_filled(draw, 32, 42, 14, 7, 'red')
    draw_ellipse_filled(draw, 22, 52, 18, 6, 'red_dark')
    draw_ellipse_filled(draw, 32, 38, 4, 3, 'red_dark')
    return img

@hat('bowler')
def draw_bowler_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 52, 22, 6, 'black')
    draw_ellipse_filled(draw, 32, 40, 14, 12, 'black')
    draw_ellipse_filled(draw, 32, 38, 10, 8, (50, 50, 60))
    draw.rectangle([18, 46, 46, 50], fill=PALETTE['black'])
    return img

@hat('archer')
def draw_archer_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 44, 20, 12, 'green')
    draw_ellipse_filled(draw, 32, 42, 16, 9, 'green_light')
    for i in range(20):
        draw_pixel(draw, 48 + i // 3, 40 - i // 2, 'red')
    return img

@hat('ninja')
def draw_ninja_headband():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([10, 42, 54, 52], fill=PALETTE['black'])
    draw.rectangle([12, 44, 52, 50], fill=(50, 50, 60))
    for i in range(16):
        draw_pixel(draw, 52 + i // 2, 46 + (i % 4) - 2, 'black')
        draw_pixel(draw, 54 + i // 2, 48 + (i % 4) - 2, 'black')
    draw_ellipse_filled(draw, 32, 47, 6, 4, 'silver')
    return img

@hat('mushroom')
def draw_mushroom_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 38, 24, 18, 'red')
    draw_ellipse_filled(draw, 32, 40, 20, 14, 'red')
    for x, y in [(20, 32), (44, 32), (28, 26), (36, 26), (32, 44)]:
        draw_ellipse_filled(draw, x, y, 5, 5, 'white')
    draw.rectangle([26, 50, 38, 58], fill=PALETTE['white'])
    return img

@hat('detective')
def draw_detective_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 50, 20, 8, 'brown')
    draw_ellipse_filled(draw, 32, 38, 16, 14, 'brown')
    draw_ellipse_filled(draw, 32, 36, 12, 10, 'brown_light')
    draw.rectangle([16, 44, 48, 50], fill=PALETTE['brown_dark'])
    return img

@hat('alien_antenna')
def draw_alien_antenna():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.arc([18, 48, 46, 58], 180, 360, fill=PALETTE['green'], width=3)
    draw.line([26, 50, 20, 20], fill=PALETTE['green'], width=2)
    draw.line([38, 50, 44, 20], fill=PALETTE['green'], width=2)
    draw_ellipse_filled(draw, 20, 16, 5, 5, 'green_light')
    draw_ellipse_filled(draw, 44, 16, 5, 5, 'green_light')
    return img

@hat('jester')
def draw_jester_hat():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 50, 20, 8, 'mid_purple')
    for i, (dx, color) in enumerate([(-18, 'red'), (0, 'gold'), (18, 'blue')]):
        for y in range(22):
            w = int(6 - y * 0.2)
            if w > 0:
                draw.line([32 + dx - w, 48 - y, 32 + dx + w, 48 - y], fill=PALETTE[color])
        draw_ellipse_filled(draw, 32 + dx, 26, 4, 4, 'gold')
    return img

@hat('unicorn_horn')
def draw_unicorn_horn():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.arc([18, 48, 46, 58], 180, 360, fill=PALETTE['pink'], width=3)
    colors = ['pink', 'gold', 'cyan', 'pink', 'gold']
    for y in range(28):
        w = int(8 - y * 0.25)
        if w > 0:
            color = PALETTE[colors[(y // 4) % len(colors)]]
            draw.line([32 - w, 50 - y, 32 + w, 50 - y], fill=color)
    return img

@hat('space_helmet')
def draw_space_helmet():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 38, 22, 20, 'white')
    draw_ellipse_filled(draw, 32, 36, 18, 16, 'gray_light')
    draw_ellipse_filled(draw, 32, 34, 14, 12, 'blue_light')
    draw_ellipse_filled(draw, 26, 30, 4, 4, (200, 230, 255, 200))
    return img

# ============================================================
# HELD ITEM SPRITES (30 items)
# ============================================================

HELD_GENERATORS = {}

def held(name):
    def decorator(func):
        HELD_GENERATORS[name] = func
        return func
    return decorator

@held('book')
def draw_book():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([18, 20, 46, 50], fill=PALETTE['red_dark'])
    draw.rectangle([20, 22, 44, 48], fill=PALETTE['red'])
    draw.rectangle([22, 24, 42, 46], fill=PALETTE['white'])
    for y in range(28, 44, 3):
        draw.line([24, y, 40, y], fill=(200, 200, 210), width=1)
    draw.rectangle([18, 20, 22, 50], fill=PALETTE['red_dark'])
    draw.rectangle([36, 18, 40, 26], fill=PALETTE['gold'])
    return img

@held('wand')
def draw_wand():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.line([20, 50, 44, 20], fill=PALETTE['brown'], width=4)
    draw.line([22, 48, 46, 18], fill=PALETTE['brown_light'], width=2)
    star_x, star_y = 46, 16
    draw_pixel(draw, star_x, star_y - 4, 'gold', 2)
    draw_pixel(draw, star_x - 4, star_y, 'gold', 2)
    draw_pixel(draw, star_x + 2, star_y, 'gold', 2)
    draw_pixel(draw, star_x - 2, star_y + 3, 'gold', 2)
    draw_pixel(draw, star_x + 1, star_y + 3, 'gold', 2)
    draw_pixel(draw, star_x, star_y, 'white', 2)
    for sx, sy in [(50, 12), (40, 8), (52, 20)]:
        draw_pixel(draw, sx, sy, 'white')
    return img

@held('coffee')
def draw_coffee():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([20, 28, 44, 54], fill=PALETTE['white'])
    draw.rectangle([22, 30, 42, 52], fill=(240, 235, 230))
    draw_ellipse_filled(draw, 32, 32, 9, 4, 'brown')
    draw.arc([40, 34, 52, 50], -60, 60, fill=PALETTE['white'], width=4)
    for i, sx in enumerate([28, 32, 36]):
        for j in range(3):
            sy = 22 - j * 4 - (i % 2) * 2
            draw_pixel(draw, sx + (j % 2) * 2, sy, (200, 200, 200, 150 - j * 40), 2)
    return img

@held('pencil')
def draw_pencil():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.polygon([(22, 48), (22, 24), (42, 24), (42, 48)], fill=PALETTE['gold'])
    draw.rectangle([24, 26, 40, 46], fill=(255, 220, 120))
    draw.polygon([(22, 48), (42, 48), (32, 58)], fill=PALETTE['brown_light'])
    draw.polygon([(28, 52), (36, 52), (32, 58)], fill=PALETTE['black'])
    draw.rectangle([22, 18, 42, 24], fill=PALETTE['silver'])
    draw.rectangle([22, 14, 42, 18], fill=PALETTE['pink'])
    draw.line([22, 20, 42, 20], fill=PALETTE['silver_dark'], width=1)
    draw.line([22, 22, 42, 22], fill=PALETTE['silver_dark'], width=1)
    return img

@held('trophy')
def draw_trophy():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 24, 14, 10, 'gold')
    draw_ellipse_filled(draw, 32, 22, 10, 6, 'gold_dark')
    draw_ellipse_filled(draw, 32, 20, 10, 6, 'gold')
    draw.arc([10, 18, 22, 32], 90, 270, fill=PALETTE['gold'], width=4)
    draw.arc([42, 18, 54, 32], 270, 90, fill=PALETTE['gold'], width=4)
    draw.rectangle([28, 32, 36, 44], fill=PALETTE['gold'])
    draw.rectangle([30, 34, 34, 42], fill=PALETTE['gold_dark'])
    draw.rectangle([22, 44, 42, 48], fill=PALETTE['gold'])
    draw.rectangle([18, 48, 46, 54], fill=PALETTE['gold_dark'])
    draw.rectangle([20, 50, 44, 52], fill=PALETTE['gold'])
    draw_pixel(draw, 31, 22, 'white', 3)
    return img

@held('controller')
def draw_controller():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rounded_rectangle([12, 24, 52, 48], radius=8, fill=PALETTE['dark_purple'])
    draw.rounded_rectangle([14, 26, 50, 46], radius=6, fill=PALETTE['mid_purple'])
    draw.rectangle([18, 32, 28, 36], fill=PALETTE['black'])
    draw.rectangle([21, 29, 25, 39], fill=PALETTE['black'])
    for bx, by, color in [(40, 30, 'red'), (46, 34, 'blue'), (40, 38, 'green'), (34, 34, 'gold')]:
        draw_ellipse_filled(draw, bx, by, 3, 3, color)
    draw_ellipse_filled(draw, 26, 42, 4, 4, 'black')
    draw_ellipse_filled(draw, 38, 42, 4, 4, 'black')
    return img

@held('sword')
def draw_sword():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for i in range(28):
        w = max(1, 4 - i // 8)
        draw.line([32 - w, 12 + i, 32 + w, 12 + i], fill=PALETTE['silver'])
    draw.line([30, 12, 30, 38], fill=PALETTE['silver_light'], width=1)
    draw.rectangle([24, 40, 40, 44], fill=PALETTE['gold'])
    draw.rectangle([30, 44, 34, 56], fill=PALETTE['brown'])
    draw_ellipse_filled(draw, 32, 56, 4, 3, 'gold')
    return img

@held('shield')
def draw_shield():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 34, 18, 22, 'blue')
    draw_ellipse_filled(draw, 32, 32, 14, 18, 'blue_light')
    draw_ellipse_filled(draw, 32, 34, 8, 10, 'gold')
    draw_pixel(draw, 30, 32, 'white', 2)
    return img

@held('potion_red')
def draw_potion_red():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([28, 16, 36, 24], fill=PALETTE['brown'])
    draw_ellipse_filled(draw, 32, 24, 5, 3, 'brown')
    draw_ellipse_filled(draw, 32, 40, 12, 16, 'red')
    draw_ellipse_filled(draw, 28, 36, 6, 8, 'red_light')
    draw_pixel(draw, 26, 34, 'white', 2)
    return img

@held('potion_blue')
def draw_potion_blue():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([28, 16, 36, 24], fill=PALETTE['brown'])
    draw_ellipse_filled(draw, 32, 24, 5, 3, 'brown')
    draw_ellipse_filled(draw, 32, 40, 12, 16, 'blue')
    draw_ellipse_filled(draw, 28, 36, 6, 8, 'blue_light')
    draw_pixel(draw, 26, 34, 'white', 2)
    return img

@held('potion_green')
def draw_potion_green():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([28, 16, 36, 24], fill=PALETTE['brown'])
    draw_ellipse_filled(draw, 32, 24, 5, 3, 'brown')
    draw_ellipse_filled(draw, 32, 40, 12, 16, 'green')
    draw_ellipse_filled(draw, 28, 36, 6, 8, 'green_light')
    draw_pixel(draw, 26, 34, 'white', 2)
    return img

@held('scroll')
def draw_scroll():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([22, 20, 42, 52], fill=PALETTE['white'])
    draw.rectangle([24, 22, 40, 50], fill=(250, 245, 230))
    for y in range(26, 48, 4):
        draw.line([26, y, 38, y], fill=(180, 170, 150), width=1)
    draw_ellipse_filled(draw, 22, 22, 4, 6, (250, 245, 230))
    draw_ellipse_filled(draw, 42, 52, 4, 6, (250, 245, 230))
    return img

@held('gem_ruby')
def draw_gem_ruby():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.polygon([(32, 16), (48, 32), (32, 56), (16, 32)], fill=PALETTE['red'])
    draw.polygon([(32, 20), (44, 32), (32, 50), (20, 32)], fill=PALETTE['red_light'])
    draw.polygon([(32, 24), (38, 32), (32, 44), (26, 32)], fill=(255, 200, 200))
    draw_pixel(draw, 28, 28, 'white', 3)
    return img

@held('gem_sapphire')
def draw_gem_sapphire():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.polygon([(32, 16), (48, 32), (32, 56), (16, 32)], fill=PALETTE['blue'])
    draw.polygon([(32, 20), (44, 32), (32, 50), (20, 32)], fill=PALETTE['blue_light'])
    draw.polygon([(32, 24), (38, 32), (32, 44), (26, 32)], fill=(200, 220, 255))
    draw_pixel(draw, 28, 28, 'white', 3)
    return img

@held('gem_emerald')
def draw_gem_emerald():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.polygon([(32, 16), (48, 32), (32, 56), (16, 32)], fill=PALETTE['green'])
    draw.polygon([(32, 20), (44, 32), (32, 50), (20, 32)], fill=PALETTE['green_light'])
    draw.polygon([(32, 24), (38, 32), (32, 44), (26, 32)], fill=(200, 255, 220))
    draw_pixel(draw, 28, 28, 'white', 3)
    return img

@held('lantern')
def draw_lantern():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([26, 14, 38, 18], fill=PALETTE['gold'])
    draw.arc([28, 10, 36, 18], 0, 180, fill=PALETTE['gold'], width=2)
    draw.rectangle([24, 18, 40, 52], fill=PALETTE['gold_dark'])
    draw.rectangle([26, 20, 38, 50], fill=PALETTE['orange'])
    draw_ellipse_filled(draw, 32, 36, 4, 8, 'yellow')
    draw_pixel(draw, 30, 32, 'white', 2)
    return img

@held('magnifier')
def draw_magnifier():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.line([38, 42, 52, 56], fill=PALETTE['brown'], width=4)
    draw.ellipse([16, 16, 44, 44], outline=PALETTE['gold'], width=4)
    draw_ellipse_filled(draw, 30, 30, 10, 10, (200, 220, 255, 150))
    draw.arc([22, 22, 34, 34], 200, 340, fill=PALETTE['white'], width=2)
    return img

@held('compass')
def draw_compass():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.ellipse([16, 16, 48, 48], outline=PALETTE['gold'], width=3)
    draw_ellipse_filled(draw, 32, 32, 14, 14, 'white')
    draw.polygon([(32, 20), (36, 32), (32, 44), (28, 32)], fill=PALETTE['red'])
    draw.polygon([(32, 32), (36, 32), (32, 44), (28, 32)], fill=PALETTE['black'])
    draw_ellipse_filled(draw, 32, 32, 3, 3, 'gold')
    return img

@held('hourglass')
def draw_hourglass():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([22, 14, 42, 18], fill=PALETTE['gold'])
    draw.rectangle([22, 50, 42, 54], fill=PALETTE['gold'])
    draw.polygon([(24, 18), (40, 18), (32, 34)], fill=PALETTE['cyan'])
    draw.polygon([(24, 50), (40, 50), (32, 34)], fill=PALETTE['cyan'])
    draw_ellipse_filled(draw, 32, 44, 6, 4, 'gold_light')
    return img

@held('quill')
def draw_quill():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for i in range(30):
        x = 20 + i
        y = 50 - i
        w = max(1, 6 - i // 6)
        draw.line([x, y - w, x, y + w], fill=PALETTE['white'])
    draw.line([20, 50, 26, 56], fill=PALETTE['brown'], width=2)
    draw_pixel(draw, 26, 56, 'black')
    return img

@held('crystal_ball')
def draw_crystal_ball():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 30, 16, 16, 'mid_purple')
    draw_ellipse_filled(draw, 28, 26, 8, 8, 'light_purple')
    draw_pixel(draw, 24, 22, 'white', 3)
    draw.polygon([(20, 46), (44, 46), (48, 54), (16, 54)], fill=PALETTE['gold'])
    return img

@held('dice')
def draw_dice():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([20, 20, 44, 44], fill=PALETTE['white'])
    draw.rectangle([22, 22, 42, 42], fill=(250, 250, 255))
    for x, y in [(26, 26), (32, 32), (38, 38), (26, 38), (38, 26)]:
        draw_ellipse_filled(draw, x, y, 2, 2, 'black')
    return img

@held('key')
def draw_key():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 22, 10, 10, 'gold')
    draw_ellipse_filled(draw, 32, 22, 5, 5, 'gold_dark')
    draw.rectangle([30, 30, 34, 54], fill=PALETTE['gold'])
    draw.rectangle([34, 48, 40, 52], fill=PALETTE['gold'])
    draw.rectangle([34, 40, 38, 44], fill=PALETTE['gold'])
    return img

@held('heart')
def draw_heart_item():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 24, 28, 10, 10, 'red')
    draw_ellipse_filled(draw, 40, 28, 10, 10, 'red')
    draw.polygon([(14, 32), (32, 54), (50, 32)], fill=PALETTE['red'])
    draw_ellipse_filled(draw, 22, 26, 4, 4, 'red_light')
    draw_pixel(draw, 20, 24, 'white', 2)
    return img

@held('star')
def draw_star_item():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    points = []
    import math
    for i in range(10):
        angle = math.radians(i * 36 - 90)
        r = 20 if i % 2 == 0 else 10
        points.append((32 + r * math.cos(angle), 32 + r * math.sin(angle)))
    draw.polygon(points, fill=PALETTE['gold'])
    draw_ellipse_filled(draw, 28, 28, 4, 4, 'gold_light')
    draw_pixel(draw, 26, 26, 'white', 2)
    return img

@held('music_note')
def draw_music_note():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 26, 46, 8, 6, 'black')
    draw_ellipse_filled(draw, 42, 42, 8, 6, 'black')
    draw.rectangle([32, 20, 36, 46], fill=PALETTE['black'])
    draw.rectangle([48, 16, 52, 42], fill=PALETTE['black'])
    draw.rectangle([36, 16, 52, 22], fill=PALETTE['black'])
    return img

@held('paintbrush')
def draw_paintbrush():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([30, 16, 34, 42], fill=PALETTE['brown'])
    draw.rectangle([28, 42, 36, 48], fill=PALETTE['silver'])
    draw_ellipse_filled(draw, 32, 52, 6, 6, 'red')
    draw_ellipse_filled(draw, 30, 50, 3, 3, 'red_light')
    return img

@held('telescope')
def draw_telescope():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.line([18, 50, 46, 22], fill=PALETTE['brown'], width=6)
    draw.line([20, 48, 48, 20], fill=PALETTE['brown_light'], width=4)
    draw_ellipse_filled(draw, 48, 20, 6, 6, 'gold')
    draw_ellipse_filled(draw, 48, 20, 4, 4, 'blue_light')
    draw_ellipse_filled(draw, 18, 50, 4, 4, 'gold')
    return img

@held('bell')
def draw_bell():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw_ellipse_filled(draw, 32, 22, 4, 4, 'gold')
    draw.arc([32, 18, 36, 26], 0, 180, fill=PALETTE['gold'], width=2)
    draw_ellipse_filled(draw, 32, 38, 14, 16, 'gold')
    draw_ellipse_filled(draw, 32, 36, 10, 12, 'gold_light')
    draw_ellipse_filled(draw, 32, 52, 4, 4, 'gold_dark')
    return img

@held('feather')
def draw_feather():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    # Feather quill
    draw.line([32, 54, 38, 16], fill=PALETTE['white'], width=2)
    # Feather body
    for i in range(20):
        w = max(1, 8 - abs(i - 10) // 2)
        y = 20 + i
        draw.line([38 - i // 3 - w, y, 38 - i // 3 + w, y], fill=PALETTE['cyan'])
    for i in range(14):
        w = max(1, 4 - abs(i - 7) // 2)
        y = 24 + i
        draw.line([38 - i // 3 - w + 2, y, 38 - i // 3 + w + 2, y], fill=PALETTE['cyan_dark'])
    return img

# ============================================================
# AURA SPRITES (10+ auras)
# ============================================================

AURA_GENERATORS = {}

def aura(name):
    def decorator(func):
        AURA_GENERATORS[name] = func
        return func
    return decorator

@aura('sparkles')
def draw_sparkle_aura():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    positions = [(8, 12), (56, 8), (4, 40), (60, 44), (12, 56), (52, 58), (32, 4), (48, 24), (16, 28), (44, 52)]
    for x, y in positions:
        size = 2 + (x + y) % 2
        color = (255, 255, 200, 180)
        draw_pixel(draw, x, y, color, 2)
        draw_pixel(draw, x - size, y, color)
        draw_pixel(draw, x + size, y, color)
        draw_pixel(draw, x, y - size, color)
        draw_pixel(draw, x, y + size, color)
    return img

@aura('hearts')
def draw_hearts_aura():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    positions = [(10, 10), (54, 16), (8, 48), (56, 52), (30, 6), (48, 38)]
    for hx, hy in positions:
        size = 3 + (hx % 2)
        color = (255, 150, 180, 200)
        draw_ellipse_filled(draw, hx - size//2, hy, size, size, color)
        draw_ellipse_filled(draw, hx + size//2, hy, size, size, color)
        for i in range(size + 1):
            w = size - i
            draw.line([hx - w, hy + i + size//2, hx + w, hy + i + size//2], fill=color)
    return img

@aura('stars')
def draw_stars_aura():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    positions = [(6, 8), (58, 12), (12, 52), (50, 56), (28, 4), (36, 60), (4, 30), (60, 34)]
    for sx, sy in positions:
        size = 2 + (sx + sy) % 2
        color = (255, 255, 150, 200)
        draw_pixel(draw, sx, sy, color, 2)
        for i in range(1, size + 1):
            alpha = 200 - i * 40
            c = (255, 255, 150, max(alpha, 50))
            draw_pixel(draw, sx - i, sy, c)
            draw_pixel(draw, sx + i, sy, c)
            draw_pixel(draw, sx, sy - i, c)
            draw_pixel(draw, sx, sy + i, c)
    return img

@aura('flames')
def draw_flame_aura():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    random.seed(789)
    bases = [(8, 58), (20, 60), (32, 62), (44, 60), (56, 58)]
    colors = [(255, 100, 50, 180), (255, 150, 50, 160), (255, 200, 100, 140), (255, 255, 200, 100)]
    for fx, fy in bases:
        height = 12 + (fx % 4) * 2
        for i in range(height):
            width = max(1, 4 - i // 3)
            color_idx = min(i // 3, len(colors) - 1)
            for w in range(-width, width + 1):
                y = fy - i
                x = fx + w + ((i % 3) - 1)
                if 0 <= x < 64 and 0 <= y < 64:
                    draw_pixel(draw, x, y, colors[color_idx])
    return img

@aura('rainbow')
def draw_rainbow_aura():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    colors = [(255, 100, 100, 100), (255, 180, 100, 100), (255, 255, 100, 100), (100, 255, 100, 100), (100, 200, 255, 100), (150, 100, 255, 100), (200, 100, 255, 100)]
    for i, color in enumerate(colors):
        r = 28 - i * 2
        if r > 0:
            draw.arc([32 - r, 32 - r, 32 + r, 32 + r], 0, 360, fill=color, width=2)
    return img

@aura('bubbles')
def draw_bubbles_aura():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    random.seed(321)
    for _ in range(12):
        x, y = random.randint(4, 60), random.randint(4, 60)
        r = random.randint(3, 8)
        draw.ellipse([x - r, y - r, x + r, y + r], outline=(150, 200, 255, 150), width=1)
        draw_pixel(draw, x - r//2, y - r//2, (255, 255, 255, 200))
    return img

@aura('leaves')
def draw_leaves_aura():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    random.seed(654)
    for _ in range(10):
        x, y = random.randint(4, 60), random.randint(4, 60)
        draw_ellipse_filled(draw, x, y, 4, 2, (100, 200, 100, 180))
        draw.line([x, y, x + 3, y + 3], fill=(80, 160, 80, 180), width=1)
    return img

@aura('snow')
def draw_snow_aura():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    random.seed(111)
    for _ in range(20):
        x, y = random.randint(2, 62), random.randint(2, 62)
        size = random.randint(1, 3)
        draw_pixel(draw, x, y, (255, 255, 255, 200), size)
    return img

@aura('lightning')
def draw_lightning_aura():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    color = (255, 255, 100, 200)
    for start_x in [16, 48]:
        x, y = start_x, 4
        for _ in range(8):
            nx = x + random.randint(-4, 4)
            ny = y + random.randint(4, 8)
            draw.line([x, y, nx, ny], fill=color, width=2)
            x, y = nx, ny
    return img

@aura('music')
def draw_music_aura():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    positions = [(8, 12), (56, 20), (12, 48), (52, 44), (28, 8), (40, 56)]
    for x, y in positions:
        draw_ellipse_filled(draw, x, y + 4, 3, 2, (200, 150, 255, 180))
        draw.rectangle([x + 2, y - 4, x + 4, y + 4], fill=(200, 150, 255, 180))
    return img

@aura('coins')
def draw_coins_aura():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    positions = [(10, 8), (54, 14), (8, 52), (56, 48), (32, 6), (20, 58), (44, 58)]
    for x, y in positions:
        draw_ellipse_filled(draw, x, y, 5, 5, (255, 215, 100, 200))
        draw_ellipse_filled(draw, x - 1, y - 1, 2, 2, (255, 240, 180, 200))
    return img

@aura('magic')
def draw_magic_aura():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    import math
    for i in range(8):
        angle = math.radians(i * 45)
        for r in range(20, 32, 4):
            x = 32 + int(r * math.cos(angle))
            y = 32 + int(r * math.sin(angle))
            alpha = 180 - (r - 20) * 10
            draw_pixel(draw, x, y, (180, 100, 255, alpha), 2)
    return img

# ============================================================
# BACKGROUND SPRITES (30 backgrounds)
# ============================================================

BG_GENERATORS = {}

def bg(name):
    def decorator(func):
        BG_GENERATORS[name] = func
        return func
    return decorator

@bg('starfield')
def draw_starfield_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        color = (20 + y // 4, 15 + y // 5, 35 + y // 3, 255)
        draw.line([0, y, 63, y], fill=color)
    random.seed(42)
    for _ in range(30):
        x, y = random.randint(0, 63), random.randint(0, 50)
        brightness = random.randint(150, 255)
        size = 1 if random.random() > 0.3 else 2
        draw_pixel(draw, x, y, (brightness, brightness, brightness - 20, 255), size)
    return img

@bg('cozy_room')
def draw_cozy_room_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([0, 0, 63, 40], fill=(60, 50, 80))
    draw.rectangle([0, 40, 63, 63], fill=(80, 60, 50))
    for x in range(0, 64, 12):
        draw.line([x, 40, x, 63], fill=(60, 45, 35), width=1)
    draw.rectangle([20, 8, 44, 30], fill=(100, 120, 150))
    draw.rectangle([22, 10, 42, 28], fill=(150, 180, 220))
    draw.line([32, 10, 32, 28], fill=(80, 70, 90), width=2)
    draw.line([22, 19, 42, 19], fill=(80, 70, 90), width=2)
    draw.rectangle([50, 12, 60, 24], fill=(100, 70, 50))
    draw.rectangle([52, 14, 58, 22], fill=(180, 150, 200))
    return img

@bg('garden')
def draw_garden_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(40):
        blue = 150 + (40 - y)
        draw.line([0, y, 63, y], fill=(120, 180, min(255, blue)))
    draw.rectangle([0, 40, 63, 63], fill=(80, 160, 90))
    random.seed(123)
    for _ in range(40):
        x = random.randint(0, 63)
        h = random.randint(3, 8)
        draw.line([x, 40, x, 40 - h], fill=(60, 140, 70), width=1)
    for fx, fy, color in [(10, 48, 'red'), (25, 52, 'gold'), (45, 50, 'blue'), (55, 54, 'pink')]:
        draw_ellipse_filled(draw, fx, fy, 3, 3, color)
        draw.line([fx, fy + 3, fx, fy + 8], fill=(50, 120, 60), width=2)
    draw_ellipse_filled(draw, 54, 10, 8, 8, 'gold')
    return img

@bg('library')
def draw_library_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([0, 0, 63, 63], fill=(50, 40, 60))
    draw.rectangle([4, 8, 60, 56], fill=(80, 50, 40))
    for y in [20, 36, 52]:
        draw.rectangle([6, y, 58, y + 2], fill=(60, 35, 25))
    colors = ['red', 'blue', 'green', 'gold', 'mid_purple']
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

@bg('clouds')
def draw_clouds_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        purple = 120 + y
        blue = 150 + y // 2
        draw.line([0, y, 63, y], fill=(min(255, purple), 140, min(255, blue)))
    positions = [(10, 15), (35, 25), (55, 12), (20, 40), (45, 45)]
    for cx, cy in positions:
        for dx, dy, r in [(0, 0, 8), (-6, 2, 5), (6, 2, 5), (-3, -3, 4), (3, -3, 4)]:
            draw_ellipse_filled(draw, cx + dx, cy + dy, r, r // 2 + 2, (255, 255, 255, 180))
    return img

@bg('sunset')
def draw_sunset_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        r = min(255, 255 - y * 2)
        g = max(0, 150 - y * 3)
        b = min(255, 100 + y * 2)
        draw.line([0, y, 63, y], fill=(r, g, b))
    draw_ellipse_filled(draw, 50, 20, 12, 12, 'gold')
    draw_ellipse_filled(draw, 48, 18, 6, 6, 'yellow')
    return img

@bg('ocean')
def draw_ocean_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        b = max(80, 200 - y * 2)
        draw.line([0, y, 63, y], fill=(40, 80 + y // 2, b))
    random.seed(777)
    for _ in range(8):
        x = random.randint(0, 60)
        y = random.randint(10, 50)
        draw.arc([x, y, x + 12, y + 4], 0, 180, fill=(100, 150, 200, 150), width=1)
    return img

@bg('forest')
def draw_forest_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        draw.line([0, y, 63, y], fill=(30 + y // 4, 60 + y // 3, 40 + y // 4))
    random.seed(888)
    for _ in range(8):
        x = random.randint(5, 58)
        h = random.randint(20, 40)
        draw.rectangle([x - 2, 60 - h, x + 2, 60], fill=(60, 40, 30))
        draw.polygon([(x, 60 - h - 15), (x - 10, 60 - h + 5), (x + 10, 60 - h + 5)], fill=(40, 100, 50))
    return img

@bg('desert')
def draw_desert_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(30):
        draw.line([0, y, 63, y], fill=(180, 200, 255 - y))
    for y in range(30, 64):
        draw.line([0, y, 63, y], fill=(220, 180 - (y - 30), 100))
    draw_ellipse_filled(draw, 50, 12, 8, 8, 'gold')
    draw.polygon([(20, 55), (30, 35), (40, 55)], fill=(200, 160, 80))
    draw.polygon([(35, 55), (42, 40), (50, 55)], fill=(180, 140, 70))
    return img

@bg('cave')
def draw_cave_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([0, 0, 63, 63], fill=(30, 25, 40))
    random.seed(999)
    for _ in range(20):
        x, y = random.randint(0, 63), random.randint(0, 63)
        draw_pixel(draw, x, y, (50, 45, 60), random.randint(2, 4))
    for x in [12, 32, 52]:
        draw.polygon([(x - 6, 0), (x, 12), (x + 6, 0)], fill=(50, 45, 60))
        draw.polygon([(x - 4, 63), (x, 55), (x + 4, 63)], fill=(50, 45, 60))
    draw_ellipse_filled(draw, 32, 32, 6, 6, (80, 150, 200, 100))
    return img

@bg('castle')
def draw_castle_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        draw.line([0, y, 63, y], fill=(80, 70, 100 + y // 2))
    draw.rectangle([20, 30, 44, 60], fill=(60, 55, 70))
    for x in [20, 26, 32, 38, 44]:
        draw.rectangle([x - 2, 24, x + 2, 30], fill=(70, 65, 80))
    draw.rectangle([28, 45, 36, 60], fill=(40, 35, 50))
    draw_ellipse_filled(draw, 32, 45, 4, 4, (40, 35, 50))
    for x in [8, 56]:
        draw.rectangle([x - 4, 35, x + 4, 60], fill=(60, 55, 70))
        draw.rectangle([x - 2, 28, x + 2, 35], fill=(70, 65, 80))
    return img

@bg('space')
def draw_space_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([0, 0, 63, 63], fill=(10, 5, 20))
    random.seed(1234)
    for _ in range(50):
        x, y = random.randint(0, 63), random.randint(0, 63)
        b = random.randint(100, 255)
        draw_pixel(draw, x, y, (b, b, b - 20), 1)
    draw_ellipse_filled(draw, 48, 16, 10, 10, (100, 80, 120))
    draw_ellipse_filled(draw, 50, 14, 4, 4, (80, 60, 100))
    return img

@bg('underwater')
def draw_underwater_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        draw.line([0, y, 63, y], fill=(20, 60 + y // 2, 120 + y // 2))
    random.seed(2345)
    for _ in range(15):
        x, y = random.randint(0, 63), random.randint(0, 63)
        draw_ellipse_filled(draw, x, y, 2, 3, (150, 200, 255, 100))
    for x in [10, 30, 50]:
        for i in range(8):
            draw.line([x, 58 - i * 2, x + random.randint(-2, 2), 58 - i * 2 - 4], fill=(80, 180, 100), width=2)
    return img

@bg('candy')
def draw_candy_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    colors = [(255, 200, 220), (200, 220, 255), (220, 255, 200), (255, 220, 200)]
    for y in range(64):
        draw.line([0, y, 63, y], fill=colors[(y // 8) % len(colors)])
    for _ in range(8):
        x, y = random.randint(5, 58), random.randint(5, 58)
        draw_ellipse_filled(draw, x, y, 4, 4, random.choice(['pink', 'cyan', 'yellow', 'green_light']))
    return img

@bg('volcanic')
def draw_volcanic_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        r = min(80, 40 + y // 2)
        draw.line([0, y, 63, y], fill=(r, 20, 30))
    draw.polygon([(20, 60), (32, 25), (44, 60)], fill=(60, 40, 45))
    draw_ellipse_filled(draw, 32, 28, 6, 4, (255, 100, 50))
    for i in range(5):
        x = 32 + random.randint(-4, 4)
        y = 20 - i * 3
        draw_pixel(draw, x, y, (255, 150, 50), 2)
    return img

@bg('arctic')
def draw_arctic_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        b = min(255, 200 + y // 2)
        draw.line([0, y, 63, y], fill=(220, 230, b))
    draw.polygon([(10, 60), (20, 40), (30, 60)], fill=(240, 245, 255))
    draw.polygon([(35, 60), (50, 35), (60, 60)], fill=(230, 240, 250))
    random.seed(3456)
    for _ in range(15):
        x, y = random.randint(0, 63), random.randint(0, 30)
        draw_pixel(draw, x, y, (255, 255, 255), 1)
    return img

@bg('jungle')
def draw_jungle_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        draw.line([0, y, 63, y], fill=(40, 80 + y // 2, 50))
    random.seed(4567)
    for _ in range(12):
        x = random.randint(0, 60)
        draw.line([x, 0, x, 20], fill=(60, 40, 30), width=2)
        for i in range(5):
            draw.line([x, 4 + i * 4, x + random.randint(-8, 8), 6 + i * 4], fill=(80, 160, 80), width=2)
    return img

@bg('mountain')
def draw_mountain_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        draw.line([0, y, 63, y], fill=(100, 140, 200 - y))
    draw.polygon([(0, 60), (20, 30), (40, 60)], fill=(80, 80, 90))
    draw.polygon([(30, 60), (50, 25), (63, 60)], fill=(100, 100, 110))
    draw.polygon([(50, 25), (45, 35), (55, 35)], fill=(255, 255, 255))
    return img

@bg('meadow')
def draw_meadow_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(30):
        draw.line([0, y, 63, y], fill=(150, 200, 255 - y))
    for y in range(30, 64):
        draw.line([0, y, 63, y], fill=(100, 180, 100))
    random.seed(5678)
    for _ in range(20):
        x, y = random.randint(0, 63), random.randint(32, 60)
        color = random.choice(['yellow', 'pink', 'white', 'red'])
        draw_pixel(draw, x, y, color, 2)
    return img

@bg('rainy')
def draw_rainy_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        draw.line([0, y, 63, y], fill=(60, 70, 90))
    random.seed(6789)
    for _ in range(30):
        x, y = random.randint(0, 63), random.randint(0, 60)
        draw.line([x, y, x + 1, y + 4], fill=(150, 170, 200), width=1)
    return img

@bg('autumn')
def draw_autumn_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        draw.line([0, y, 63, y], fill=(200, 150 - y // 2, 100))
    random.seed(7890)
    for _ in range(15):
        x, y = random.randint(0, 63), random.randint(0, 63)
        color = random.choice(['orange', 'red', 'gold', 'brown'])
        draw_ellipse_filled(draw, x, y, 3, 2, color)
    return img

@bg('cherry_blossom')
def draw_cherry_blossom_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        draw.line([0, y, 63, y], fill=(255, 220 - y // 2, 230 - y // 3))
    random.seed(8901)
    for _ in range(25):
        x, y = random.randint(0, 63), random.randint(0, 50)
        draw_ellipse_filled(draw, x, y, 3, 3, (255, 180, 200, 200))
    return img

@bg('neon_city')
def draw_neon_city_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        draw.line([0, y, 63, y], fill=(20, 10, 40))
    random.seed(9012)
    for x in range(0, 64, 8):
        h = random.randint(15, 40)
        draw.rectangle([x, 60 - h, x + 6, 60], fill=(30, 20, 50))
        color = random.choice(['cyan', 'pink', 'yellow', 'green_light'])
        for wy in range(60 - h + 2, 58, 4):
            draw_pixel(draw, x + 2, wy, color, 2)
    return img

@bg('galaxy')
def draw_galaxy_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        for x in range(64):
            dist = ((x - 32) ** 2 + (y - 32) ** 2) ** 0.5
            r = int(max(0, 60 - dist))
            g = int(max(0, 30 - dist // 2))
            b = int(max(0, 80 - dist // 2))
            draw_pixel(draw, x, y, (r, g, b))
    random.seed(1111)
    for _ in range(40):
        x, y = random.randint(0, 63), random.randint(0, 63)
        draw_pixel(draw, x, y, (255, 255, 255), 1)
    return img

@bg('crystal_cave')
def draw_crystal_cave_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([0, 0, 63, 63], fill=(30, 35, 50))
    colors = ['cyan', 'pink', 'light_purple', 'blue_light']
    random.seed(2222)
    for _ in range(10):
        x, y = random.randint(5, 58), random.randint(5, 58)
        color = random.choice(colors)
        h = random.randint(8, 16)
        draw.polygon([(x, y - h), (x - 4, y + 4), (x + 4, y + 4)], fill=PALETTE.get(color, PALETTE['cyan']))
    return img

@bg('enchanted')
def draw_enchanted_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(64):
        draw.line([0, y, 63, y], fill=(40 + y // 3, 30 + y // 4, 60 + y // 2))
    random.seed(3333)
    for _ in range(12):
        x, y = random.randint(0, 63), random.randint(0, 63)
        draw_ellipse_filled(draw, x, y, 2, 2, (200, 255, 200, 150))
    draw_ellipse_filled(draw, 50, 12, 6, 6, (255, 255, 200, 200))
    return img

@bg('cozy_fireplace')
def draw_cozy_fireplace_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([0, 0, 63, 63], fill=(50, 35, 30))
    draw.rectangle([16, 30, 48, 60], fill=(40, 30, 25))
    draw.rectangle([18, 32, 46, 58], fill=(30, 20, 15))
    for i in range(5):
        x = 26 + i * 3
        h = 8 + (i % 2) * 4
        draw.polygon([(x, 56), (x - 2, 56 - h), (x + 2, 56 - h)], fill=PALETTE['orange'])
        draw.polygon([(x, 56), (x - 1, 56 - h + 2), (x + 1, 56 - h + 2)], fill=PALETTE['yellow'])
    return img

@bg('spring_meadow')
def draw_spring_meadow_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    for y in range(28):
        draw.line([0, y, 63, y], fill=(180, 220, 255 - y))
    for y in range(28, 64):
        draw.line([0, y, 63, y], fill=(120, 200, 120))
    random.seed(4444)
    for _ in range(30):
        x, y = random.randint(0, 63), random.randint(30, 62)
        color = random.choice(['yellow', 'pink', 'white'])
        draw_ellipse_filled(draw, x, y, 2, 2, color)
        draw_ellipse_filled(draw, x, y, 1, 1, 'yellow')
    return img

@bg('mystical_portal')
def draw_mystical_portal_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    draw.rectangle([0, 0, 63, 63], fill=(20, 15, 35))
    for r in range(24, 4, -2):
        alpha = 50 + (24 - r) * 8
        color = (100 + r * 4, 50 + r * 2, 200, alpha)
        draw.ellipse([32 - r, 32 - r, 32 + r, 32 + r], outline=color, width=2)
    draw_ellipse_filled(draw, 32, 32, 6, 6, (255, 255, 255, 200))
    return img

@bg('beach')
def draw_beach_bg():
    img = create_canvas()
    draw = ImageDraw.Draw(img)
    # Sky
    for y in range(28):
        draw.line([0, y, 63, y], fill=(135, 200, 255 - y))
    # Ocean
    for y in range(28, 42):
        draw.line([0, y, 63, y], fill=(60, 120, 180 - (y - 28)))
    # Sand
    for y in range(42, 64):
        draw.line([0, y, 63, y], fill=(240, 220, 180))
    # Sun
    draw_ellipse_filled(draw, 52, 12, 8, 8, 'gold')
    # Waves
    draw.arc([0, 36, 20, 44], 0, 180, fill=(100, 160, 200), width=2)
    draw.arc([18, 38, 38, 46], 0, 180, fill=(100, 160, 200), width=2)
    draw.arc([36, 36, 56, 44], 0, 180, fill=(100, 160, 200), width=2)
    return img

# ============================================================
# MAIN GENERATION
# ============================================================

def ensure_dir(path):
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
    print("Generating creature sprites...")
    for creature_name, draw_func in creatures.items():
        creature_dir = os.path.join(base_dir, creature_name)
        ensure_dir(creature_dir)
        for mood in moods:
            img = draw_func(mood)
            filepath = os.path.join(creature_dir, f'{mood}.png')
            img.save(filepath)
            print(f'  {creature_name}/{mood}.png')

    # Generate hat sprites
    print("\nGenerating hat sprites...")
    hats_dir = os.path.join(base_dir, 'hats')
    ensure_dir(hats_dir)
    for hat_name, draw_func in HAT_GENERATORS.items():
        img = draw_func()
        filepath = os.path.join(hats_dir, f'{hat_name}.png')
        img.save(filepath)
        print(f'  hats/{hat_name}.png')

    # Generate held item sprites
    print("\nGenerating held item sprites...")
    held_dir = os.path.join(base_dir, 'held')
    ensure_dir(held_dir)
    for item_name, draw_func in HELD_GENERATORS.items():
        img = draw_func()
        filepath = os.path.join(held_dir, f'{item_name}.png')
        img.save(filepath)
        print(f'  held/{item_name}.png')

    # Generate aura sprites
    print("\nGenerating aura sprites...")
    auras_dir = os.path.join(base_dir, 'auras')
    ensure_dir(auras_dir)
    for aura_name, draw_func in AURA_GENERATORS.items():
        img = draw_func()
        filepath = os.path.join(auras_dir, f'{aura_name}.png')
        img.save(filepath)
        print(f'  auras/{aura_name}.png')

    # Generate background sprites
    print("\nGenerating background sprites...")
    backgrounds_dir = os.path.join(base_dir, 'backgrounds')
    ensure_dir(backgrounds_dir)
    for bg_name, draw_func in BG_GENERATORS.items():
        img = draw_func()
        filepath = os.path.join(backgrounds_dir, f'{bg_name}.png')
        img.save(filepath)
        print(f'  backgrounds/{bg_name}.png')

    print('\n' + '='*50)
    print('All sprites generated successfully!')
    print(f'  Creatures: {len(creatures) * len(moods)} sprites')
    print(f'  Hats: {len(HAT_GENERATORS)} sprites')
    print(f'  Held items: {len(HELD_GENERATORS)} sprites')
    print(f'  Auras: {len(AURA_GENERATORS)} sprites')
    print(f'  Backgrounds: {len(BG_GENERATORS)} sprites')
    print(f'  TOTAL: {len(creatures) * len(moods) + len(HAT_GENERATORS) + len(HELD_GENERATORS) + len(AURA_GENERATORS) + len(BG_GENERATORS)} sprites')

if __name__ == '__main__':
    main()
