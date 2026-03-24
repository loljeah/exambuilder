#!/usr/bin/env python3
"""Generate hat SVG assets for exambuilder"""

import os

def svg_header(w=64, h=64):
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">'

def svg_footer():
    return '</svg>'

HATS = {
    'wizard': '''<defs><linearGradient id="wiz" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#6d28d9"/><stop offset="100%" stop-color="#4c1d95"/></linearGradient></defs>
    <path d="M32,2 L48,54 L16,54 Z" fill="url(#wiz)"/>
    <ellipse cx="32" cy="54" rx="20" ry="6" fill="#4c1d95"/>
    <circle cx="28" cy="20" r="2" fill="#fbbf24"/><circle cx="36" cy="32" r="3" fill="#fbbf24"/>
    <circle cx="24" cy="38" r="1.5" fill="#fbbf24"/>''',

    'crown': '''<defs><linearGradient id="gold" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fcd34d"/><stop offset="100%" stop-color="#d97706"/></linearGradient></defs>
    <path d="M8,54 L8,28 L20,38 L32,20 L44,38 L56,28 L56,54 Z" fill="url(#gold)"/>
    <circle cx="32" cy="24" r="4" fill="#ef4444"/><circle cx="18" cy="36" r="3" fill="#3b82f6"/>
    <circle cx="46" cy="36" r="3" fill="#10b981"/><rect x="8" y="50" width="48" height="4" fill="#b45309"/>''',

    'party': '''<defs><linearGradient id="party" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#ec4899"/><stop offset="100%" stop-color="#be185d"/></linearGradient></defs>
    <path d="M32,4 L50,54 L14,54 Z" fill="url(#party)"/>
    <circle cx="32" cy="4" r="4" fill="#fbbf24"/>
    <path d="M20,20 L26,40" stroke="#fbbf24" stroke-width="2"/>
    <path d="M38,24 L44,44" stroke="#22d3ee" stroke-width="2"/>
    <path d="M28,30 L32,48" stroke="#a3e635" stroke-width="2"/>''',

    'tophat': '''<defs><linearGradient id="top" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#374151"/><stop offset="100%" stop-color="#111827"/></linearGradient></defs>
    <rect x="18" y="14" width="28" height="36" rx="2" fill="url(#top)"/>
    <ellipse cx="32" cy="14" rx="14" ry="4" fill="#374151"/>
    <ellipse cx="32" cy="50" rx="22" ry="6" fill="#111827"/>
    <rect x="18" y="40" width="28" height="4" fill="#8b5cf6"/>''',

    'catears': '''<defs><linearGradient id="ear" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fce7f3"/><stop offset="100%" stop-color="#f9a8d4"/></linearGradient></defs>
    <path d="M10,52 L6,16 L28,36 Z" fill="url(#ear)"/>
    <path d="M54,52 L58,16 L36,36 Z" fill="url(#ear)"/>
    <path d="M12,48 L10,22 L24,38 Z" fill="#fbcfe8"/>
    <path d="M52,48 L54,22 L40,38 Z" fill="#fbcfe8"/>''',

    'halo': '''<defs><filter id="glow"><feGaussianBlur stdDeviation="2" result="b"/>
    <feMerge><feMergeNode in="b"/><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter></defs>
    <ellipse cx="32" cy="16" rx="20" ry="6" fill="none" stroke="#fcd34d" stroke-width="5" filter="url(#glow)"/>''',

    'beret': '''<defs><linearGradient id="beret" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#dc2626"/><stop offset="100%" stop-color="#991b1b"/></linearGradient></defs>
    <ellipse cx="32" cy="40" rx="26" ry="12" fill="url(#beret)"/>
    <ellipse cx="28" cy="32" rx="20" ry="14" fill="#dc2626"/>
    <circle cx="32" cy="18" r="4" fill="#991b1b"/>''',

    'beanie': '''<defs><linearGradient id="bean" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#3b82f6"/><stop offset="100%" stop-color="#1d4ed8"/></linearGradient></defs>
    <path d="M10,48 Q10,16 32,12 Q54,16 54,48 Z" fill="url(#bean)"/>
    <rect x="10" y="44" width="44" height="8" fill="#1e40af"/>
    <circle cx="32" cy="8" r="5" fill="#60a5fa"/>''',

    'cowboy': '''<defs><linearGradient id="cow" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#a16207"/><stop offset="100%" stop-color="#713f12"/></linearGradient></defs>
    <ellipse cx="32" cy="48" rx="30" ry="8" fill="url(#cow)"/>
    <path d="M14,48 Q14,24 32,20 Q50,24 50,48 Z" fill="#92400e"/>
    <rect x="20" y="38" width="24" height="4" fill="#78350f"/>''',

    'chef': '''<ellipse cx="32" cy="48" rx="22" ry="6" fill="#f5f5f5"/>
    <ellipse cx="32" cy="28" rx="18" ry="18" fill="white"/>
    <ellipse cx="22" cy="22" rx="8" ry="8" fill="white"/>
    <ellipse cx="42" cy="22" rx="8" ry="8" fill="white"/>
    <ellipse cx="32" cy="14" rx="10" ry="8" fill="white"/>''',

    'pirate': '''<defs><linearGradient id="pir" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#1f2937"/><stop offset="100%" stop-color="#030712"/></linearGradient></defs>
    <ellipse cx="32" cy="50" rx="26" ry="6" fill="url(#pir)"/>
    <path d="M8,50 Q8,20 32,16 Q56,20 56,50 Z" fill="#111827"/>
    <ellipse cx="32" cy="28" rx="10" ry="8" fill="white"/>
    <path d="M26,24 L38,32 M26,32 L38,24" stroke="#111827" stroke-width="2"/>''',

    'viking': '''<defs><linearGradient id="vik" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#78716c"/><stop offset="100%" stop-color="#44403c"/></linearGradient></defs>
    <ellipse cx="32" cy="44" rx="24" ry="12" fill="url(#vik)"/>
    <path d="M12,44 Q12,20 32,16 Q52,20 52,44 Z" fill="#57534e"/>
    <path d="M4,44 Q4,28 4,20" stroke="#fef3c7" stroke-width="6" stroke-linecap="round"/>
    <path d="M60,44 Q60,28 60,20" stroke="#fef3c7" stroke-width="6" stroke-linecap="round"/>''',

    'propeller': '''<rect x="28" y="40" width="8" height="16" fill="#9ca3af"/>
    <ellipse cx="32" cy="40" rx="4" ry="3" fill="#6b7280"/>
    <g transform-origin="32 40"><animateTransform attributeName="transform" type="rotate" values="0 32 40;360 32 40" dur="0.5s" repeatCount="indefinite"/>
    <ellipse cx="32" cy="28" rx="4" ry="14" fill="#3b82f6" transform="rotate(0 32 40)"/>
    <ellipse cx="32" cy="28" rx="4" ry="14" fill="#ef4444" transform="rotate(120 32 40)"/>
    <ellipse cx="32" cy="28" rx="4" ry="14" fill="#22c55e" transform="rotate(240 32 40)"/></g>''',

    'bunny_ears': '''<defs><linearGradient id="bun" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fef3c7"/><stop offset="100%" stop-color="#fde68a"/></linearGradient></defs>
    <ellipse cx="16" cy="28" rx="8" ry="24" fill="url(#bun)"/>
    <ellipse cx="48" cy="28" rx="8" ry="24" fill="url(#bun)"/>
    <ellipse cx="16" cy="28" rx="4" ry="16" fill="#fca5a5"/>
    <ellipse cx="48" cy="28" rx="4" ry="16" fill="#fca5a5"/>''',

    'santa': '''<defs><linearGradient id="santa" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#ef4444"/><stop offset="100%" stop-color="#b91c1c"/></linearGradient></defs>
    <path d="M12,52 Q12,24 32,8 Q56,20 52,52 Z" fill="url(#santa)"/>
    <ellipse cx="32" cy="52" rx="22" ry="6" fill="white"/>
    <circle cx="52" cy="12" r="8" fill="white"/>''',

    'graduation': '''<rect x="8" y="28" width="48" height="6" fill="#1f2937"/>
    <rect x="16" y="16" width="32" height="16" fill="#111827"/>
    <rect x="30" y="28" width="4" height="20" fill="#fbbf24"/>
    <circle cx="32" cy="48" r="4" fill="#fbbf24"/>''',

    'fez': '''<defs><linearGradient id="fez" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#dc2626"/><stop offset="100%" stop-color="#7f1d1d"/></linearGradient></defs>
    <path d="M16,52 Q16,24 32,20 Q48,24 48,52 Z" fill="url(#fez)"/>
    <ellipse cx="32" cy="52" rx="16" ry="4" fill="#7f1d1d"/>
    <line x1="32" y1="20" x2="32" y2="8" stroke="#111827" stroke-width="2"/>
    <path d="M32,8 Q40,16 44,24" stroke="#fbbf24" stroke-width="3" fill="none"/>''',

    'headphones': '''<path d="M8,40 Q8,16 32,12 Q56,16 56,40" fill="none" stroke="#374151" stroke-width="4"/>
    <rect x="4" y="36" width="12" height="20" rx="4" fill="#1f2937"/>
    <rect x="48" y="36" width="12" height="20" rx="4" fill="#1f2937"/>
    <rect x="6" y="40" width="8" height="12" rx="2" fill="#6b7280"/>
    <rect x="50" y="40" width="8" height="12" rx="2" fill="#6b7280"/>''',

    'flower_crown': '''<circle cx="12" cy="36" r="6" fill="#f472b6"/>
    <circle cx="24" cy="28" r="6" fill="#fbbf24"/>
    <circle cx="40" cy="28" r="6" fill="#a78bfa"/>
    <circle cx="52" cy="36" r="6" fill="#34d399"/>
    <circle cx="32" cy="24" r="7" fill="#f87171"/>
    <circle cx="12" cy="36" r="2" fill="#fbbf24"/>
    <circle cx="24" cy="28" r="2" fill="white"/>
    <circle cx="40" cy="28" r="2" fill="#fbbf24"/>
    <circle cx="52" cy="36" r="2" fill="white"/>
    <circle cx="32" cy="24" r="3" fill="#fbbf24"/>
    <path d="M8,44 Q32,36 56,44" fill="none" stroke="#22c55e" stroke-width="3"/>''',

    'tiara': '''<defs><linearGradient id="tia" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#e0e7ff"/><stop offset="100%" stop-color="#a5b4fc"/></linearGradient></defs>
    <path d="M8,48 L16,32 L24,40 L32,20 L40,40 L48,32 L56,48 Z" fill="url(#tia)"/>
    <circle cx="32" cy="24" r="4" fill="#ec4899"/>
    <circle cx="20" cy="38" r="2" fill="#38bdf8"/>
    <circle cx="44" cy="38" r="2" fill="#38bdf8"/>''',

    'baseball': '''<defs><linearGradient id="base" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#3b82f6"/><stop offset="100%" stop-color="#1d4ed8"/></linearGradient></defs>
    <path d="M8,48 Q8,20 32,16 Q56,20 56,48 Z" fill="url(#base)"/>
    <ellipse cx="32" cy="48" rx="24" ry="6" fill="#1e40af"/>
    <path d="M4,44 L24,44" stroke="#1e40af" stroke-width="4" stroke-linecap="round"/>''',

    'bowler': '''<defs><linearGradient id="bowl" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#1f2937"/><stop offset="100%" stop-color="#030712"/></linearGradient></defs>
    <ellipse cx="32" cy="48" rx="28" ry="8" fill="url(#bowl)"/>
    <ellipse cx="32" cy="32" rx="18" ry="16" fill="#111827"/>
    <rect x="14" y="40" width="36" height="4" fill="#374151"/>''',

    'archer': '''<defs><linearGradient id="arch" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#22c55e"/><stop offset="100%" stop-color="#15803d"/></linearGradient></defs>
    <path d="M12,52 Q8,28 32,20 Q56,28 52,52 Z" fill="url(#arch)"/>
    <path d="M32,20 L32,4" stroke="#92400e" stroke-width="2"/>
    <path d="M26,4 L32,12 L38,4" fill="#dc2626"/>''',

    'ninja': '''<rect x="8" y="28" width="48" height="20" fill="#1f2937"/>
    <rect x="8" y="36" width="48" height="4" fill="#374151"/>
    <rect x="4" y="32" width="8" height="12" fill="#111827"/>
    <rect x="52" y="32" width="8" height="12" fill="#111827"/>''',

    'mushroom': '''<defs><linearGradient id="mush" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#ef4444"/><stop offset="100%" stop-color="#b91c1c"/></linearGradient></defs>
    <ellipse cx="32" cy="36" rx="28" ry="20" fill="url(#mush)"/>
    <circle cx="20" cy="28" r="6" fill="white"/>
    <circle cx="44" cy="32" r="5" fill="white"/>
    <circle cx="32" cy="20" r="4" fill="white"/>
    <rect x="24" y="52" width="16" height="8" fill="#fef3c7"/>''',

    'detective': '''<defs><linearGradient id="det" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#a16207"/><stop offset="100%" stop-color="#713f12"/></linearGradient></defs>
    <ellipse cx="32" cy="48" rx="28" ry="6" fill="#713f12"/>
    <ellipse cx="32" cy="36" rx="20" ry="14" fill="url(#det)"/>
    <path d="M12,48 L8,56" stroke="#713f12" stroke-width="3"/>
    <path d="M52,48 L56,56" stroke="#713f12" stroke-width="3"/>''',

    'alien_antenna': '''<line x1="20" y1="48" x2="20" y2="16" stroke="#22c55e" stroke-width="3"/>
    <line x1="44" y1="48" x2="44" y2="16" stroke="#22c55e" stroke-width="3"/>
    <circle cx="20" cy="12" r="6" fill="#4ade80">
    <animate attributeName="opacity" values="1;0.4;1" dur="1s" repeatCount="indefinite"/></circle>
    <circle cx="44" cy="12" r="6" fill="#4ade80">
    <animate attributeName="opacity" values="0.4;1;0.4" dur="1s" repeatCount="indefinite"/></circle>''',

    'jester': '''<path d="M4,48 L16,8 L32,32 L48,8 L60,48 Z" fill="#8b5cf6"/>
    <circle cx="16" cy="8" r="5" fill="#fbbf24"/>
    <circle cx="48" cy="8" r="5" fill="#fbbf24"/>
    <path d="M4,48 L32,40 L60,48" fill="#ec4899"/>
    <circle cx="32" cy="40" r="4" fill="#fbbf24"/>''',

    'unicorn_horn': '''<defs><linearGradient id="uni" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fef3c7"/><stop offset="33%" stop-color="#fbcfe8"/>
    <stop offset="66%" stop-color="#c4b5fd"/><stop offset="100%" stop-color="#fef3c7"/></linearGradient></defs>
    <path d="M32,4 L24,56 L40,56 Z" fill="url(#uni)"/>
    <path d="M26,20 L38,20 M24,32 L40,32 M22,44 L42,44" stroke="white" stroke-width="2" opacity="0.5"/>''',

    'space_helmet': '''<defs><linearGradient id="helm" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#f5f5f5"/><stop offset="100%" stop-color="#9ca3af"/></linearGradient>
    <linearGradient id="visor" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0%" stop-color="#3b82f6" stop-opacity="0.6"/>
    <stop offset="100%" stop-color="#1e3a8a" stop-opacity="0.8"/></linearGradient></defs>
    <ellipse cx="32" cy="36" rx="26" ry="24" fill="url(#helm)"/>
    <ellipse cx="32" cy="36" rx="20" ry="18" fill="url(#visor)"/>
    <ellipse cx="26" cy="30" rx="4" ry="3" fill="white" opacity="0.4"/>'''
}

def generate_hats():
    base = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'hats')
    os.makedirs(base, exist_ok=True)

    for name, content in HATS.items():
        svg = f'{svg_header()}\n{content}\n{svg_footer()}'
        filepath = os.path.join(base, f'{name}.svg')
        with open(filepath, 'w') as f:
            f.write(svg)
        print(f'Created hats/{name}.svg')

if __name__ == '__main__':
    generate_hats()
    print('\nHats generated!')
