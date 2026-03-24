#!/usr/bin/env python3
"""Generate held item SVG assets for exambuilder"""

import os

def svg_header(w=64, h=64):
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">'

def svg_footer():
    return '</svg>'

HELD_ITEMS = {
    'book': '''<defs><linearGradient id="cover" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#8b5cf6"/><stop offset="100%" stop-color="#6d28d9"/></linearGradient></defs>
    <rect x="12" y="8" width="40" height="48" rx="2" fill="url(#cover)"/>
    <rect x="16" y="8" width="32" height="48" fill="#fef3c7"/>
    <rect x="12" y="8" width="6" height="48" fill="#7c3aed"/>
    <line x1="22" y1="20" x2="42" y2="20" stroke="#d4d4d4" stroke-width="2"/>
    <line x1="22" y1="28" x2="42" y2="28" stroke="#d4d4d4" stroke-width="2"/>
    <line x1="22" y1="36" x2="36" y2="36" stroke="#d4d4d4" stroke-width="2"/>''',

    'wand': '''<defs><linearGradient id="wand" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#1f2937"/><stop offset="100%" stop-color="#030712"/></linearGradient>
    <filter id="glow"><feGaussianBlur stdDeviation="2"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter></defs>
    <rect x="28" y="16" width="8" height="44" rx="2" fill="url(#wand)"/>
    <rect x="26" y="52" width="12" height="6" rx="1" fill="#92400e"/>
    <circle cx="32" cy="10" r="6" fill="#fbbf24" filter="url(#glow)">
    <animate attributeName="r" values="6;8;6" dur="1.5s" repeatCount="indefinite"/></circle>''',

    'coffee': '''<defs><linearGradient id="cup" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#f5f5f5"/><stop offset="100%" stop-color="#d4d4d4"/></linearGradient></defs>
    <path d="M16,20 L20,56 L44,56 L48,20 Z" fill="url(#cup)"/>
    <ellipse cx="32" cy="20" rx="16" ry="4" fill="#e5e5e5"/>
    <ellipse cx="32" cy="24" rx="12" ry="3" fill="#78350f"/>
    <path d="M48,28 Q60,28 60,40 Q60,52 48,52" fill="none" stroke="#d4d4d4" stroke-width="4"/>
    <path d="M24,12 Q28,4 32,12" stroke="#9ca3af" stroke-width="2" fill="none" opacity="0.6">
    <animate attributeName="opacity" values="0.6;0.2;0.6" dur="2s" repeatCount="indefinite"/></path>''',

    'pencil': '''<defs><linearGradient id="pen" x1="0" y1="0" x2="1" y2="0">
    <stop offset="0%" stop-color="#fbbf24"/><stop offset="100%" stop-color="#f59e0b"/></linearGradient></defs>
    <rect x="26" y="8" width="12" height="44" fill="url(#pen)"/>
    <polygon points="26,52 38,52 32,62" fill="#fef3c7"/>
    <polygon points="30,56 34,56 32,62" fill="#374151"/>
    <rect x="26" y="8" width="12" height="6" fill="#ec4899"/>
    <rect x="26" y="14" width="12" height="4" fill="#9ca3af"/>''',

    'trophy': '''<defs><linearGradient id="gold" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fcd34d"/><stop offset="100%" stop-color="#b45309"/></linearGradient></defs>
    <path d="M20,8 L20,28 Q20,40 32,44 Q44,40 44,28 L44,8 Z" fill="url(#gold)"/>
    <path d="M20,12 Q8,12 8,24 Q8,32 20,32" fill="none" stroke="#fcd34d" stroke-width="4"/>
    <path d="M44,12 Q56,12 56,24 Q56,32 44,32" fill="none" stroke="#fcd34d" stroke-width="4"/>
    <rect x="28" y="44" width="8" height="8" fill="#92400e"/>
    <rect x="22" y="52" width="20" height="6" rx="1" fill="#b45309"/>
    <circle cx="32" cy="24" r="6" fill="#fef3c7" opacity="0.3"/>''',

    'controller': '''<defs><linearGradient id="ctrl" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#374151"/><stop offset="100%" stop-color="#1f2937"/></linearGradient></defs>
    <rect x="8" y="20" width="48" height="28" rx="8" fill="url(#ctrl)"/>
    <circle cx="20" cy="34" r="6" fill="#111827"/>
    <rect x="17" y="31" width="6" height="6" rx="1" fill="#4b5563"/>
    <circle cx="48" cy="28" r="3" fill="#ef4444"/>
    <circle cx="54" cy="34" r="3" fill="#3b82f6"/>
    <circle cx="48" cy="40" r="3" fill="#22c55e"/>
    <circle cx="42" cy="34" r="3" fill="#fbbf24"/>
    <rect x="28" y="30" width="8" height="4" rx="1" fill="#4b5563"/>''',

    'sword': '''<defs><linearGradient id="blade" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#e5e7eb"/><stop offset="100%" stop-color="#6b7280"/></linearGradient></defs>
    <path d="M28,4 L36,4 L36,36 L28,36 Z" fill="url(#blade)"/>
    <path d="M32,4 L32,36" stroke="white" stroke-width="1" opacity="0.5"/>
    <rect x="22" y="36" width="20" height="6" rx="1" fill="#92400e"/>
    <rect x="28" y="42" width="8" height="16" rx="1" fill="#78350f"/>
    <circle cx="32" cy="50" r="2" fill="#fbbf24"/>''',

    'shield': '''<defs><linearGradient id="shld" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#3b82f6"/><stop offset="100%" stop-color="#1d4ed8"/></linearGradient></defs>
    <path d="M32,4 L56,12 L56,32 Q56,56 32,60 Q8,56 8,32 L8,12 Z" fill="url(#shld)"/>
    <path d="M32,12 L48,18 L48,32 Q48,48 32,52 Q16,48 16,32 L16,18 Z" fill="#1e40af"/>
    <path d="M32,20 L32,44 M24,32 L40,32" stroke="#fbbf24" stroke-width="4"/>''',

    'potion_red': '''<defs><linearGradient id="liqr" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#f87171"/><stop offset="100%" stop-color="#b91c1c"/></linearGradient>
    <linearGradient id="glass" x1="0" y1="0" x2="1" y2="0">
    <stop offset="0%" stop-color="white" stop-opacity="0.3"/>
    <stop offset="100%" stop-color="white" stop-opacity="0"/></linearGradient></defs>
    <path d="M24,24 L20,52 Q20,58 32,58 Q44,58 44,52 L40,24 Z" fill="url(#liqr)"/>
    <rect x="26" y="8" width="12" height="16" rx="2" fill="#d4d4d4"/>
    <rect x="24" y="6" width="16" height="4" rx="1" fill="#92400e"/>
    <ellipse cx="28" cy="40" rx="4" ry="6" fill="url(#glass)"/>''',

    'potion_blue': '''<defs><linearGradient id="liqb" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#60a5fa"/><stop offset="100%" stop-color="#1d4ed8"/></linearGradient>
    <linearGradient id="glass" x1="0" y1="0" x2="1" y2="0">
    <stop offset="0%" stop-color="white" stop-opacity="0.3"/>
    <stop offset="100%" stop-color="white" stop-opacity="0"/></linearGradient></defs>
    <path d="M24,24 L20,52 Q20,58 32,58 Q44,58 44,52 L40,24 Z" fill="url(#liqb)"/>
    <rect x="26" y="8" width="12" height="16" rx="2" fill="#d4d4d4"/>
    <rect x="24" y="6" width="16" height="4" rx="1" fill="#92400e"/>
    <ellipse cx="28" cy="40" rx="4" ry="6" fill="url(#glass)"/>''',

    'potion_green': '''<defs><linearGradient id="liqg" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#4ade80"/><stop offset="100%" stop-color="#15803d"/></linearGradient>
    <linearGradient id="glass" x1="0" y1="0" x2="1" y2="0">
    <stop offset="0%" stop-color="white" stop-opacity="0.3"/>
    <stop offset="100%" stop-color="white" stop-opacity="0"/></linearGradient></defs>
    <path d="M24,24 L20,52 Q20,58 32,58 Q44,58 44,52 L40,24 Z" fill="url(#liqg)"/>
    <rect x="26" y="8" width="12" height="16" rx="2" fill="#d4d4d4"/>
    <rect x="24" y="6" width="16" height="4" rx="1" fill="#92400e"/>
    <ellipse cx="28" cy="40" rx="4" ry="6" fill="url(#glass)"/>''',

    'scroll': '''<defs><linearGradient id="parch" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fef3c7"/><stop offset="100%" stop-color="#fde68a"/></linearGradient></defs>
    <rect x="12" y="16" width="40" height="36" fill="url(#parch)"/>
    <ellipse cx="12" cy="34" rx="4" ry="18" fill="#fcd34d"/>
    <ellipse cx="52" cy="34" rx="4" ry="18" fill="#fcd34d"/>
    <line x1="20" y1="24" x2="44" y2="24" stroke="#9ca3af" stroke-width="2"/>
    <line x1="20" y1="32" x2="44" y2="32" stroke="#9ca3af" stroke-width="2"/>
    <line x1="20" y1="40" x2="36" y2="40" stroke="#9ca3af" stroke-width="2"/>''',

    'gem_ruby': '''<defs><linearGradient id="ruby" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0%" stop-color="#fca5a5"/><stop offset="50%" stop-color="#ef4444"/>
    <stop offset="100%" stop-color="#7f1d1d"/></linearGradient>
    <filter id="sparkle"><feGaussianBlur stdDeviation="1"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter></defs>
    <polygon points="32,8 52,24 44,56 20,56 12,24" fill="url(#ruby)"/>
    <polygon points="32,8 40,24 32,56 24,24" fill="#fca5a5" opacity="0.3"/>
    <polygon points="32,16 36,24 32,32 28,24" fill="white" opacity="0.4" filter="url(#sparkle)"/>''',

    'gem_sapphire': '''<defs><linearGradient id="saph" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0%" stop-color="#93c5fd"/><stop offset="50%" stop-color="#3b82f6"/>
    <stop offset="100%" stop-color="#1e3a8a"/></linearGradient>
    <filter id="sparkle"><feGaussianBlur stdDeviation="1"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter></defs>
    <polygon points="32,8 52,24 44,56 20,56 12,24" fill="url(#saph)"/>
    <polygon points="32,8 40,24 32,56 24,24" fill="#93c5fd" opacity="0.3"/>
    <polygon points="32,16 36,24 32,32 28,24" fill="white" opacity="0.4" filter="url(#sparkle)"/>''',

    'gem_emerald': '''<defs><linearGradient id="emer" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0%" stop-color="#86efac"/><stop offset="50%" stop-color="#22c55e"/>
    <stop offset="100%" stop-color="#14532d"/></linearGradient>
    <filter id="sparkle"><feGaussianBlur stdDeviation="1"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter></defs>
    <polygon points="32,8 52,24 44,56 20,56 12,24" fill="url(#emer)"/>
    <polygon points="32,8 40,24 32,56 24,24" fill="#86efac" opacity="0.3"/>
    <polygon points="32,16 36,24 32,32 28,24" fill="white" opacity="0.4" filter="url(#sparkle)"/>''',

    'lantern': '''<defs><linearGradient id="lant" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fbbf24"/><stop offset="100%" stop-color="#f59e0b"/></linearGradient>
    <filter id="flame"><feGaussianBlur stdDeviation="2"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter></defs>
    <rect x="20" y="8" width="24" height="4" rx="1" fill="#374151"/>
    <path d="M20,12 L16,52 L48,52 L44,12 Z" fill="#1f2937" opacity="0.3"/>
    <rect x="18" y="12" width="28" height="4" fill="#374151"/>
    <rect x="18" y="48" width="28" height="6" rx="1" fill="#374151"/>
    <ellipse cx="32" cy="32" rx="8" ry="10" fill="url(#lant)" filter="url(#flame)">
    <animate attributeName="ry" values="10;12;10" dur="0.5s" repeatCount="indefinite"/></ellipse>
    <path d="M28,4 Q32,0 36,4" fill="none" stroke="#374151" stroke-width="2"/>''',

    'magnifier': '''<defs><linearGradient id="lens" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0%" stop-color="#bfdbfe" stop-opacity="0.6"/>
    <stop offset="100%" stop-color="#3b82f6" stop-opacity="0.2"/></linearGradient></defs>
    <circle cx="26" cy="26" r="18" fill="url(#lens)" stroke="#92400e" stroke-width="4"/>
    <ellipse cx="20" cy="20" rx="4" ry="3" fill="white" opacity="0.5"/>
    <rect x="38" y="38" width="8" height="20" rx="2" fill="#78350f" transform="rotate(45 42 48)"/>''',

    'compass': '''<defs><linearGradient id="comp" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fcd34d"/><stop offset="100%" stop-color="#b45309"/></linearGradient></defs>
    <circle cx="32" cy="32" r="26" fill="url(#comp)" stroke="#92400e" stroke-width="2"/>
    <circle cx="32" cy="32" r="20" fill="#fef3c7"/>
    <polygon points="32,14 36,32 32,36 28,32" fill="#ef4444"/>
    <polygon points="32,50 36,32 32,28 28,32" fill="#1f2937"/>
    <circle cx="32" cy="32" r="3" fill="#92400e"/>
    <text x="32" y="18" text-anchor="middle" font-size="6" fill="#374151">N</text>''',

    'hourglass': '''<defs><linearGradient id="hour" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fcd34d"/><stop offset="100%" stop-color="#92400e"/></linearGradient></defs>
    <rect x="16" y="4" width="32" height="4" rx="1" fill="url(#hour)"/>
    <rect x="16" y="56" width="32" height="4" rx="1" fill="url(#hour)"/>
    <path d="M18,8 L18,24 Q18,32 32,32 Q46,32 46,24 L46,8 Z" fill="#bfdbfe" opacity="0.4"/>
    <path d="M18,56 L18,40 Q18,32 32,32 Q46,32 46,40 L46,56 Z" fill="#bfdbfe" opacity="0.4"/>
    <path d="M22,12 L22,22 Q22,28 32,32" fill="#fde68a"/>
    <ellipse cx="32" cy="50" rx="10" ry="4" fill="#fde68a"/>''',

    'quill': '''<defs><linearGradient id="quill" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fef3c7"/><stop offset="100%" stop-color="#fbbf24"/></linearGradient></defs>
    <path d="M44,4 Q20,20 16,56 L20,56 Q28,24 48,8 Z" fill="url(#quill)"/>
    <path d="M44,4 Q36,8 32,16" stroke="#92400e" stroke-width="1"/>
    <path d="M16,56 L18,60" stroke="#374151" stroke-width="2"/>''',

    'crystal_ball': '''<defs><radialGradient id="cryst" cx="30%" cy="30%">
    <stop offset="0%" stop-color="#c4b5fd"/><stop offset="50%" stop-color="#8b5cf6"/>
    <stop offset="100%" stop-color="#4c1d95"/></radialGradient>
    <filter id="mist"><feTurbulence type="fractalNoise" baseFrequency="0.02" numOctaves="2">
    <animate attributeName="baseFrequency" values="0.02;0.03;0.02" dur="4s" repeatCount="indefinite"/></feTurbulence>
    <feDisplacementMap in="SourceGraphic" scale="4"/></filter></defs>
    <circle cx="32" cy="28" r="22" fill="url(#cryst)"/>
    <ellipse cx="24" cy="20" rx="6" ry="4" fill="white" opacity="0.4"/>
    <rect x="20" y="50" width="24" height="8" rx="2" fill="#92400e"/>
    <ellipse cx="32" cy="28" rx="16" ry="16" fill="#a78bfa" opacity="0.2" filter="url(#mist)"/>''',

    'dice': '''<rect x="12" y="12" width="40" height="40" rx="6" fill="#f5f5f5" stroke="#d4d4d4" stroke-width="2"/>
    <circle cx="24" cy="24" r="4" fill="#1f2937"/>
    <circle cx="40" cy="24" r="4" fill="#1f2937"/>
    <circle cx="32" cy="32" r="4" fill="#1f2937"/>
    <circle cx="24" cy="40" r="4" fill="#1f2937"/>
    <circle cx="40" cy="40" r="4" fill="#1f2937"/>''',

    'key': '''<defs><linearGradient id="key" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fcd34d"/><stop offset="100%" stop-color="#92400e"/></linearGradient></defs>
    <circle cx="32" cy="16" r="12" fill="none" stroke="url(#key)" stroke-width="6"/>
    <rect x="29" y="24" width="6" height="32" fill="url(#key)"/>
    <rect x="29" y="44" width="12" height="4" fill="url(#key)"/>
    <rect x="29" y="52" width="8" height="4" fill="url(#key)"/>''',

    'heart': '''<defs><linearGradient id="heart" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fca5a5"/><stop offset="100%" stop-color="#dc2626"/></linearGradient>
    <filter id="pulse"><feGaussianBlur stdDeviation="1"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter></defs>
    <path d="M32,56 Q8,36 8,20 Q8,8 20,8 Q28,8 32,16 Q36,8 44,8 Q56,8 56,20 Q56,36 32,56 Z" fill="url(#heart)" filter="url(#pulse)">
    <animate attributeName="transform" type="scale" values="1;1.05;1" dur="0.8s" repeatCount="indefinite" additive="sum"/></path>
    <ellipse cx="20" cy="18" rx="4" ry="3" fill="white" opacity="0.4"/>''',

    'star': '''<defs><linearGradient id="star" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fef08a"/><stop offset="100%" stop-color="#fbbf24"/></linearGradient>
    <filter id="glow"><feGaussianBlur stdDeviation="2"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter></defs>
    <polygon points="32,4 38,24 60,24 42,38 50,58 32,46 14,58 22,38 4,24 26,24" fill="url(#star)" filter="url(#glow)"/>
    <polygon points="32,12 36,24 48,24 38,32 42,44 32,36 22,44 26,32 16,24 28,24" fill="#fef3c7" opacity="0.4"/>''',

    'music_note': '''<defs><linearGradient id="note" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#8b5cf6"/><stop offset="100%" stop-color="#6d28d9"/></linearGradient></defs>
    <ellipse cx="20" cy="48" rx="10" ry="8" fill="url(#note)"/>
    <rect x="28" y="12" width="4" height="40" fill="url(#note)"/>
    <path d="M32,12 Q48,8 48,24" fill="none" stroke="url(#note)" stroke-width="4"/>''',

    'paintbrush': '''<defs><linearGradient id="brush" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#a16207"/><stop offset="100%" stop-color="#713f12"/></linearGradient></defs>
    <rect x="28" y="8" width="8" height="32" rx="1" fill="url(#brush)"/>
    <rect x="26" y="36" width="12" height="4" fill="#9ca3af"/>
    <path d="M26,40 Q20,52 24,60 L32,56 L40,60 Q44,52 38,40 Z" fill="#ec4899"/>
    <path d="M28,44 L32,54" stroke="#be185d" stroke-width="2"/>''',

    'telescope': '''<defs><linearGradient id="tele" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#92400e"/><stop offset="100%" stop-color="#713f12"/></linearGradient></defs>
    <rect x="8" y="28" width="20" height="12" rx="2" fill="#fcd34d"/>
    <rect x="26" y="26" width="24" height="16" rx="2" fill="url(#tele)"/>
    <rect x="48" y="24" width="12" height="20" rx="2" fill="#78350f"/>
    <ellipse cx="56" cy="34" rx="4" ry="8" fill="#bfdbfe" opacity="0.4"/>''',

    'bell': '''<defs><linearGradient id="bell" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fcd34d"/><stop offset="100%" stop-color="#b45309"/></linearGradient></defs>
    <path d="M32,8 Q16,8 16,32 L16,48 L48,48 L48,32 Q48,8 32,8 Z" fill="url(#bell)"/>
    <rect x="12" y="48" width="40" height="4" rx="1" fill="#92400e"/>
    <circle cx="32" cy="56" r="4" fill="#92400e"/>
    <rect x="30" y="4" width="4" height="8" rx="1" fill="#78350f"/>
    <ellipse cx="24" cy="24" rx="4" ry="6" fill="white" opacity="0.3"/>''',

    'feather': '''<defs><linearGradient id="feath" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0%" stop-color="#a78bfa"/><stop offset="100%" stop-color="#6d28d9"/></linearGradient></defs>
    <path d="M48,4 Q32,16 20,40 Q16,52 20,60 L24,56 Q28,44 36,28 Q44,16 48,4 Z" fill="url(#feath)"/>
    <path d="M48,4 Q36,20 24,52" stroke="#4c1d95" stroke-width="1.5" fill="none"/>
    <path d="M36,16 L44,12 M32,24 L42,18 M28,32 L38,26 M24,40 L34,34" stroke="#c4b5fd" stroke-width="1"/>'''
}

def generate_held():
    base = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'held')
    os.makedirs(base, exist_ok=True)

    for name, content in HELD_ITEMS.items():
        svg = f'{svg_header()}\n{content}\n{svg_footer()}'
        filepath = os.path.join(base, f'{name}.svg')
        with open(filepath, 'w') as f:
            f.write(svg)
        print(f'Created held/{name}.svg')

if __name__ == '__main__':
    generate_held()
    print('\nHeld items generated!')
