#!/usr/bin/env python3
"""
Vector Art Asset Generator for ExamBuilder
Generates SVG assets with cozy purple theme
"""

import os
import math

# Cozy Purple Palette (matching exambuilder theme)
PALETTE = {
    'bg_dark': '#1a1625',
    'bg_mid': '#2d2640',
    'bg_light': '#3d3555',
    'primary': '#8b5cf6',
    'primary_light': '#a78bfa',
    'primary_dark': '#6d28d9',
    'accent_gold': '#f59e0b',
    'accent_pink': '#ec4899',
    'accent_green': '#10b981',
    'accent_red': '#ef4444',
    'accent_blue': '#3b82f6',
    'text_light': '#f5f5f5',
    'text_muted': '#9ca3af',
}

# Creature color schemes
CREATURES = {
    'cat': {'body': '#f5d0c5', 'dark': '#d4a99a', 'accent': '#ff9999', 'eyes': '#4a5568'},
    'slime': {'body': '#a78bfa', 'dark': '#7c3aed', 'accent': '#c4b5fd', 'eyes': '#1e1b4b'},
    'octopus': {'body': '#f472b6', 'dark': '#db2777', 'accent': '#fbcfe8', 'eyes': '#1e1b4b'},
    'snail': {'body': '#6ee7b7', 'dark': '#059669', 'accent': '#a7f3d0', 'eyes': '#1e1b4b'},
}

MOODS = ['happy', 'content', 'neutral', 'sad', 'lonely']

def svg_header(w=64, h=64):
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">'''

def svg_footer():
    return '</svg>'

def breathing_style():
    return '''<style>
@keyframes breathe{0%,100%{transform:scaleY(1)}50%{transform:scaleY(1.03)}}
.body{animation:breathe 2s ease-in-out infinite;transform-origin:center bottom}
</style>'''

# ============ CREATURE GENERATORS ============

def generate_cat(mood):
    c = CREATURES['cat']
    # Mood affects eyes and mouth
    eye_happy = '<path d="M20,28 Q24,32 28,28" stroke="#4a5568" fill="none" stroke-width="2" stroke-linecap="round"/><path d="M36,28 Q40,32 44,28" stroke="#4a5568" fill="none" stroke-width="2" stroke-linecap="round"/>'
    eye_content = '<ellipse cx="24" cy="28" rx="3" ry="4" fill="#4a5568"/><ellipse cx="40" cy="28" rx="3" ry="4" fill="#4a5568"/><circle cx="25" cy="27" r="1" fill="white"/><circle cx="41" cy="27" r="1" fill="white"/>'
    eye_neutral = '<ellipse cx="24" cy="28" rx="3" ry="3" fill="#4a5568"/><ellipse cx="40" cy="28" rx="3" ry="3" fill="#4a5568"/><circle cx="25" cy="27" r="1" fill="white"/><circle cx="41" cy="27" r="1" fill="white"/>'
    eye_sad = '<ellipse cx="24" cy="30" rx="3" ry="4" fill="#4a5568"/><ellipse cx="40" cy="30" rx="3" ry="4" fill="#4a5568"/><path d="M20,25 L28,27" stroke="#4a5568" stroke-width="1.5"/><path d="M44,25 L36,27" stroke="#4a5568" stroke-width="1.5"/>'
    eye_lonely = '<ellipse cx="24" cy="30" rx="2" ry="3" fill="#4a5568"/><ellipse cx="40" cy="30" rx="2" ry="3" fill="#4a5568"/><circle cx="38" cy="38" r="3" fill="#88ccff" opacity="0.6"/>'

    eyes = {'happy': eye_happy, 'content': eye_content, 'neutral': eye_neutral, 'sad': eye_sad, 'lonely': eye_lonely}[mood]

    mouth_happy = '<path d="M28,38 Q32,44 36,38" stroke="#4a5568" fill="none" stroke-width="1.5" stroke-linecap="round"/>'
    mouth_content = '<path d="M29,38 Q32,40 35,38" stroke="#4a5568" fill="none" stroke-width="1.5" stroke-linecap="round"/>'
    mouth_neutral = '<line x1="29" y1="38" x2="35" y2="38" stroke="#4a5568" stroke-width="1.5" stroke-linecap="round"/>'
    mouth_sad = '<path d="M28,40 Q32,36 36,40" stroke="#4a5568" fill="none" stroke-width="1.5" stroke-linecap="round"/>'
    mouth_lonely = '<path d="M29,40 Q32,37 35,40" stroke="#4a5568" fill="none" stroke-width="1.5" stroke-linecap="round"/>'

    mouth = {'happy': mouth_happy, 'content': mouth_content, 'neutral': mouth_neutral, 'sad': mouth_sad, 'lonely': mouth_lonely}[mood]

    return f'''{svg_header()}
{breathing_style()}
<defs>
  <linearGradient id="catBody" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="{c['accent']}"/>
    <stop offset="100%" stop-color="{c['body']}"/>
  </linearGradient>
</defs>
<g class="body">
  <!-- Ears -->
  <path d="M16,18 L12,4 L24,14 Z" fill="{c['body']}"/>
  <path d="M48,18 L52,4 L40,14 Z" fill="{c['body']}"/>
  <path d="M17,16 L14,6 L22,14 Z" fill="{c['accent']}"/>
  <path d="M47,16 L50,6 L42,14 Z" fill="{c['accent']}"/>
  <!-- Head -->
  <ellipse cx="32" cy="32" rx="22" ry="20" fill="url(#catBody)"/>
  <!-- Cheeks -->
  <ellipse cx="16" cy="36" rx="6" ry="4" fill="{c['accent']}" opacity="0.5"/>
  <ellipse cx="48" cy="36" rx="6" ry="4" fill="{c['accent']}" opacity="0.5"/>
  <!-- Eyes -->
  {eyes}
  <!-- Nose -->
  <ellipse cx="32" cy="34" rx="2" ry="1.5" fill="{c['dark']}"/>
  <!-- Mouth -->
  {mouth}
  <!-- Whiskers -->
  <g stroke="{c['dark']}" stroke-width="0.5" opacity="0.6">
    <line x1="8" y1="32" x2="18" y2="34"/>
    <line x1="8" y1="36" x2="18" y2="36"/>
    <line x1="56" y1="32" x2="46" y2="34"/>
    <line x1="56" y1="36" x2="46" y2="36"/>
  </g>
</g>
{svg_footer()}'''


def generate_slime(mood):
    c = CREATURES['slime']

    eye_happy = '<path d="M20,26 Q24,30 28,26" stroke="#1e1b4b" fill="none" stroke-width="2.5" stroke-linecap="round"/><path d="M36,26 Q40,30 44,26" stroke="#1e1b4b" fill="none" stroke-width="2.5" stroke-linecap="round"/>'
    eye_content = '<ellipse cx="24" cy="26" rx="4" ry="5" fill="#1e1b4b"/><ellipse cx="40" cy="26" rx="4" ry="5" fill="#1e1b4b"/><ellipse cx="25" cy="24" r="2" fill="white"/><ellipse cx="41" cy="24" r="2" fill="white"/>'
    eye_neutral = '<ellipse cx="24" cy="26" rx="4" ry="4" fill="#1e1b4b"/><ellipse cx="40" cy="26" rx="4" ry="4" fill="#1e1b4b"/><ellipse cx="25" cy="25" r="1.5" fill="white"/><ellipse cx="41" cy="25" r="1.5" fill="white"/>'
    eye_sad = '<ellipse cx="24" cy="28" rx="4" ry="5" fill="#1e1b4b"/><ellipse cx="40" cy="28" rx="4" ry="5" fill="#1e1b4b"/><path d="M20,23 L28,26" stroke="#1e1b4b" stroke-width="2"/><path d="M44,23 L36,26" stroke="#1e1b4b" stroke-width="2"/>'
    eye_lonely = '<ellipse cx="24" cy="28" rx="3" ry="4" fill="#1e1b4b"/><ellipse cx="40" cy="28" rx="3" ry="4" fill="#1e1b4b"/><ellipse cx="42" cy="36" rx="4" ry="3" fill="#88ccff" opacity="0.5"/>'

    eyes = {'happy': eye_happy, 'content': eye_content, 'neutral': eye_neutral, 'sad': eye_sad, 'lonely': eye_lonely}[mood]

    mouth_happy = '<path d="M26,38 Q32,46 38,38" stroke="#1e1b4b" fill="none" stroke-width="2" stroke-linecap="round"/>'
    mouth_content = '<path d="M28,38 Q32,42 36,38" stroke="#1e1b4b" fill="none" stroke-width="2" stroke-linecap="round"/>'
    mouth_neutral = '<ellipse cx="32" cy="38" rx="3" ry="2" fill="#1e1b4b"/>'
    mouth_sad = '<path d="M26,42 Q32,36 38,42" stroke="#1e1b4b" fill="none" stroke-width="2" stroke-linecap="round"/>'
    mouth_lonely = '<path d="M28,40 Q32,36 36,40" stroke="#1e1b4b" fill="none" stroke-width="1.5" stroke-linecap="round"/>'

    mouth = {'happy': mouth_happy, 'content': mouth_content, 'neutral': mouth_neutral, 'sad': mouth_sad, 'lonely': mouth_lonely}[mood]

    return f'''{svg_header()}
{breathing_style()}
<defs>
  <linearGradient id="slimeBody" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="{c['accent']}"/>
    <stop offset="100%" stop-color="{c['dark']}"/>
  </linearGradient>
  <filter id="slimeGlow" x="-20%" y="-20%" width="140%" height="140%">
    <feGaussianBlur in="SourceAlpha" stdDeviation="2"/>
    <feOffset dx="0" dy="0"/>
    <feComposite in2="SourceAlpha" operator="arithmetic" k2="-1" k3="1"/>
    <feColorMatrix values="0 0 0 0 0.7  0 0 0 0 0.4  0 0 0 0 1  0 0 0 0.5 0"/>
    <feBlend in2="SourceGraphic"/>
  </filter>
</defs>
<g class="body" filter="url(#slimeGlow)">
  <!-- Body blob -->
  <path d="M8,52 Q8,20 32,16 Q56,20 56,52 Q44,56 32,54 Q20,56 8,52 Z" fill="url(#slimeBody)"/>
  <!-- Highlight -->
  <ellipse cx="22" cy="24" rx="6" ry="4" fill="white" opacity="0.3"/>
  <!-- Eyes -->
  {eyes}
  <!-- Mouth -->
  {mouth}
</g>
{svg_footer()}'''


def generate_octopus(mood):
    c = CREATURES['octopus']

    eye_happy = '<path d="M18,24 Q22,28 26,24" stroke="#1e1b4b" fill="none" stroke-width="2" stroke-linecap="round"/><path d="M38,24 Q42,28 46,24" stroke="#1e1b4b" fill="none" stroke-width="2" stroke-linecap="round"/>'
    eye_content = '<ellipse cx="22" cy="24" rx="4" ry="5" fill="#1e1b4b"/><ellipse cx="42" cy="24" rx="4" ry="5" fill="#1e1b4b"/><circle cx="23" cy="22" r="2" fill="white"/><circle cx="43" cy="22" r="2" fill="white"/>'
    eye_neutral = '<ellipse cx="22" cy="24" rx="4" ry="4" fill="#1e1b4b"/><ellipse cx="42" cy="24" rx="4" ry="4" fill="#1e1b4b"/><circle cx="23" cy="23" r="1.5" fill="white"/><circle cx="43" cy="23" r="1.5" fill="white"/>'
    eye_sad = '<ellipse cx="22" cy="26" rx="4" ry="5" fill="#1e1b4b"/><ellipse cx="42" cy="26" rx="4" ry="5" fill="#1e1b4b"/><path d="M18,21 L26,24" stroke="#1e1b4b" stroke-width="2"/><path d="M46,21 L38,24" stroke="#1e1b4b" stroke-width="2"/>'
    eye_lonely = '<ellipse cx="22" cy="26" rx="3" ry="4" fill="#1e1b4b"/><ellipse cx="42" cy="26" rx="3" ry="4" fill="#1e1b4b"/><ellipse cx="44" cy="34" rx="3" ry="2" fill="#88ccff" opacity="0.5"/>'

    eyes = {'happy': eye_happy, 'content': eye_content, 'neutral': eye_neutral, 'sad': eye_sad, 'lonely': eye_lonely}[mood]

    mouth_happy = '<path d="M28,34 Q32,40 36,34" stroke="#1e1b4b" fill="none" stroke-width="1.5" stroke-linecap="round"/>'
    mouth_content = '<path d="M29,34 Q32,37 35,34" stroke="#1e1b4b" fill="none" stroke-width="1.5" stroke-linecap="round"/>'
    mouth_neutral = '<ellipse cx="32" cy="34" rx="2" ry="1.5" fill="#1e1b4b"/>'
    mouth_sad = '<path d="M28,36 Q32,32 36,36" stroke="#1e1b4b" fill="none" stroke-width="1.5" stroke-linecap="round"/>'
    mouth_lonely = '<path d="M29,36 Q32,33 35,36" stroke="#1e1b4b" fill="none" stroke-width="1.5" stroke-linecap="round"/>'

    mouth = {'happy': mouth_happy, 'content': mouth_content, 'neutral': mouth_neutral, 'sad': mouth_sad, 'lonely': mouth_lonely}[mood]

    return f'''{svg_header()}
{breathing_style()}
<defs>
  <linearGradient id="octoBody" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="{c['accent']}"/>
    <stop offset="100%" stop-color="{c['dark']}"/>
  </linearGradient>
</defs>
<g class="body">
  <!-- Tentacles -->
  <g fill="{c['body']}">
    <path d="M8,40 Q4,52 8,60 Q12,56 10,44 Z"/>
    <path d="M16,44 Q10,56 16,62 Q20,56 18,48 Z"/>
    <path d="M24,46 Q20,58 26,62 Q30,56 26,50 Z"/>
    <path d="M40,46 Q36,58 42,62 Q46,56 42,50 Z" transform="scale(-1,1) translate(-64,0)"/>
    <path d="M48,44 Q42,56 48,62 Q52,56 50,48 Z" transform="scale(-1,1) translate(-64,0)"/>
    <path d="M56,40 Q52,52 56,60 Q60,56 58,44 Z" transform="scale(-1,1) translate(-64,0)"/>
  </g>
  <!-- Head -->
  <ellipse cx="32" cy="28" rx="24" ry="22" fill="url(#octoBody)"/>
  <!-- Spots -->
  <circle cx="14" cy="30" r="3" fill="{c['accent']}" opacity="0.5"/>
  <circle cx="50" cy="30" r="3" fill="{c['accent']}" opacity="0.5"/>
  <circle cx="32" cy="44" r="2" fill="{c['accent']}" opacity="0.5"/>
  <!-- Eyes -->
  {eyes}
  <!-- Mouth -->
  {mouth}
</g>
{svg_footer()}'''


def generate_snail(mood):
    c = CREATURES['snail']

    eye_happy = '<path d="M22,18 Q26,22 30,18" stroke="#1e1b4b" fill="none" stroke-width="2" stroke-linecap="round"/><path d="M38,18 Q42,22 46,18" stroke="#1e1b4b" fill="none" stroke-width="2" stroke-linecap="round"/>'
    eye_content = '<ellipse cx="26" cy="18" rx="3" ry="4" fill="#1e1b4b"/><ellipse cx="42" cy="18" rx="3" ry="4" fill="#1e1b4b"/><circle cx="27" cy="17" r="1.5" fill="white"/><circle cx="43" cy="17" r="1.5" fill="white"/>'
    eye_neutral = '<ellipse cx="26" cy="18" rx="3" ry="3" fill="#1e1b4b"/><ellipse cx="42" cy="18" rx="3" ry="3" fill="#1e1b4b"/><circle cx="27" cy="17" r="1" fill="white"/><circle cx="43" cy="17" r="1" fill="white"/>'
    eye_sad = '<ellipse cx="26" cy="20" rx="3" ry="4" fill="#1e1b4b"/><ellipse cx="42" cy="20" rx="3" ry="4" fill="#1e1b4b"/><path d="M22,15 L30,18" stroke="#1e1b4b" stroke-width="1.5"/><path d="M46,15 L38,18" stroke="#1e1b4b" stroke-width="1.5"/>'
    eye_lonely = '<ellipse cx="26" cy="20" rx="2" ry="3" fill="#1e1b4b"/><ellipse cx="42" cy="20" rx="2" ry="3" fill="#1e1b4b"/><ellipse cx="44" cy="26" rx="3" ry="2" fill="#88ccff" opacity="0.5"/>'

    eyes = {'happy': eye_happy, 'content': eye_content, 'neutral': eye_neutral, 'sad': eye_sad, 'lonely': eye_lonely}[mood]

    mouth_happy = '<path d="M30,28 Q34,34 38,28" stroke="#1e1b4b" fill="none" stroke-width="1.5" stroke-linecap="round"/>'
    mouth_content = '<path d="M31,28 Q34,31 37,28" stroke="#1e1b4b" fill="none" stroke-width="1.5" stroke-linecap="round"/>'
    mouth_neutral = '<line x1="31" y1="28" x2="37" y2="28" stroke="#1e1b4b" stroke-width="1.5" stroke-linecap="round"/>'
    mouth_sad = '<path d="M30,30 Q34,26 38,30" stroke="#1e1b4b" fill="none" stroke-width="1.5" stroke-linecap="round"/>'
    mouth_lonely = '<path d="M31,30 Q34,27 37,30" stroke="#1e1b4b" fill="none" stroke-width="1.5" stroke-linecap="round"/>'

    mouth = {'happy': mouth_happy, 'content': mouth_content, 'neutral': mouth_neutral, 'sad': mouth_sad, 'lonely': mouth_lonely}[mood]

    return f'''{svg_header()}
{breathing_style()}
<defs>
  <linearGradient id="snailBody" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="{c['accent']}"/>
    <stop offset="100%" stop-color="{c['body']}"/>
  </linearGradient>
  <linearGradient id="shellGrad" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0%" stop-color="#fbbf24"/>
    <stop offset="50%" stop-color="#f59e0b"/>
    <stop offset="100%" stop-color="#d97706"/>
  </linearGradient>
</defs>
<g class="body">
  <!-- Shell -->
  <ellipse cx="20" cy="44" rx="16" ry="14" fill="url(#shellGrad)"/>
  <path d="M20,36 Q12,44 20,52 Q28,44 20,36" fill="#d97706" opacity="0.5"/>
  <circle cx="16" cy="44" r="4" fill="#fbbf24" opacity="0.6"/>
  <!-- Body -->
  <path d="M32,56 Q16,58 8,54 Q4,50 8,46 Q12,42 20,44 L20,44 Q28,42 48,44 Q56,46 56,52 Q52,58 32,56 Z" fill="url(#snailBody)"/>
  <!-- Eye stalks -->
  <path d="M26,44 Q26,30 26,12" stroke="{c['body']}" stroke-width="4" fill="none" stroke-linecap="round"/>
  <path d="M42,44 Q42,30 42,12" stroke="{c['body']}" stroke-width="4" fill="none" stroke-linecap="round"/>
  <circle cx="26" cy="12" r="5" fill="{c['body']}"/>
  <circle cx="42" cy="12" r="5" fill="{c['body']}"/>
  <!-- Eyes on stalks -->
  {eyes}
  <!-- Face on body -->
  {mouth}
</g>
{svg_footer()}'''


# ============ MAIN GENERATOR ============

def ensure_dir(path):
    os.makedirs(path, exist_ok=True)

def generate_creatures():
    base = os.path.dirname(os.path.abspath(__file__))
    generators = {
        'cat': generate_cat,
        'slime': generate_slime,
        'octopus': generate_octopus,
        'snail': generate_snail,
    }

    for creature, gen_func in generators.items():
        creature_dir = os.path.join(base, creature)
        ensure_dir(creature_dir)
        for mood in MOODS:
            svg = gen_func(mood)
            filepath = os.path.join(creature_dir, f'{mood}.svg')
            with open(filepath, 'w') as f:
                f.write(svg)
            print(f'Created {creature}/{mood}.svg')

if __name__ == '__main__':
    generate_creatures()
    print('\nCreature avatars generated!')
