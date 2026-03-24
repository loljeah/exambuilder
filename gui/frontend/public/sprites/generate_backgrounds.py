#!/usr/bin/env python3
"""Generate background SVG assets for exambuilder"""

import os

def svg_header(w=64, h=64):
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">'

def svg_footer():
    return '</svg>'

BACKGROUNDS = {
    'starfield': '''<defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#0f172a"/><stop offset="100%" stop-color="#1e1b4b"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#sky)"/>
    <circle cx="8" cy="12" r="1" fill="white"/><circle cx="24" cy="8" r="0.5" fill="white"/>
    <circle cx="48" cy="16" r="1.5" fill="#fef08a"/><circle cx="56" cy="28" r="0.5" fill="white"/>
    <circle cx="12" cy="36" r="0.5" fill="white"/><circle cx="36" cy="24" r="1" fill="white"/>
    <circle cx="52" cy="44" r="0.5" fill="white"/><circle cx="20" cy="52" r="1" fill="#c4b5fd"/>
    <circle cx="44" cy="56" r="0.5" fill="white"/><circle cx="4" cy="48" r="1" fill="white"/>''',

    'cozy_room': '''<defs><linearGradient id="wall" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#4c1d95"/><stop offset="100%" stop-color="#3b0764"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#wall)"/>
    <rect x="0" y="48" width="64" height="16" fill="#78350f"/>
    <rect x="4" y="50" width="12" height="14" fill="#92400e"/>
    <rect x="20" y="8" width="24" height="20" rx="2" fill="#1e1b4b" stroke="#fcd34d" stroke-width="2"/>
    <circle cx="32" cy="18" r="6" fill="#fef08a" opacity="0.3"/>
    <rect x="48" y="32" width="12" height="20" fill="#7c3aed"/>''',

    'garden': '''<defs><linearGradient id="gsky" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#7dd3fc"/><stop offset="100%" stop-color="#bae6fd"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#gsky)"/>
    <ellipse cx="32" cy="64" rx="40" ry="16" fill="#22c55e"/>
    <circle cx="12" cy="52" r="8" fill="#16a34a"/><circle cx="52" cy="52" r="6" fill="#16a34a"/>
    <circle cx="20" cy="44" r="4" fill="#f472b6"/><circle cx="44" cy="46" r="3" fill="#fbbf24"/>
    <circle cx="32" cy="42" r="5" fill="#a78bfa"/><circle cx="8" cy="48" r="3" fill="#f87171"/>''',

    'library': '''<defs><linearGradient id="wood" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#78350f"/><stop offset="100%" stop-color="#451a03"/></linearGradient></defs>
    <rect width="64" height="64" fill="#1c1917"/>
    <rect x="0" y="0" width="64" height="20" fill="url(#wood)"/>
    <rect x="0" y="24" width="64" height="20" fill="url(#wood)"/>
    <rect x="0" y="48" width="64" height="16" fill="url(#wood)"/>
    <g fill="#8b5cf6"><rect x="4" y="2" width="6" height="16"/><rect x="12" y="4" width="8" height="14"/><rect x="22" y="2" width="5" height="16"/></g>
    <g fill="#3b82f6"><rect x="30" y="26" width="7" height="16"/><rect x="40" y="28" width="6" height="14"/><rect x="48" y="26" width="8" height="16"/></g>''',

    'clouds': '''<defs><linearGradient id="csky" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#60a5fa"/><stop offset="100%" stop-color="#93c5fd"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#csky)"/>
    <g fill="white" opacity="0.9">
      <ellipse cx="16" cy="20" rx="12" ry="8"/><ellipse cx="24" cy="16" rx="8" ry="6"/><ellipse cx="8" cy="22" rx="6" ry="4"/>
      <ellipse cx="48" cy="36" rx="14" ry="10"/><ellipse cx="40" cy="32" rx="8" ry="6"/><ellipse cx="56" cy="38" rx="8" ry="6"/>
      <ellipse cx="28" cy="52" rx="10" ry="6"/><ellipse cx="36" cy="50" rx="6" ry="4"/>
    </g>''',

    'sunset': '''<defs><linearGradient id="sun" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fbbf24"/><stop offset="30%" stop-color="#f97316"/>
    <stop offset="60%" stop-color="#dc2626"/><stop offset="100%" stop-color="#7c3aed"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#sun)"/>
    <circle cx="32" cy="48" r="16" fill="#fcd34d" opacity="0.8"/>
    <rect x="0" y="52" width="64" height="12" fill="#1f2937" opacity="0.3"/>''',

    'ocean': '''<defs><linearGradient id="sea" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#0ea5e9"/><stop offset="100%" stop-color="#0369a1"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#sea)"/>
    <path d="M0,32 Q16,28 32,32 T64,32 V64 H0 Z" fill="#0284c7" opacity="0.5"/>
    <path d="M0,40 Q16,36 32,40 T64,40 V64 H0 Z" fill="#0369a1" opacity="0.5"/>
    <path d="M0,48 Q16,44 32,48 T64,48 V64 H0 Z" fill="#075985" opacity="0.5"/>''',

    'forest': '''<defs><linearGradient id="fsky" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#134e4a"/><stop offset="100%" stop-color="#064e3b"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#fsky)"/>
    <polygon points="8,64 16,32 24,64" fill="#166534"/><polygon points="20,64 32,24 44,64" fill="#15803d"/>
    <polygon points="36,64 48,28 60,64" fill="#166534"/><polygon points="0,64 8,40 16,64" fill="#14532d"/>
    <polygon points="48,64 56,36 64,64" fill="#14532d"/>''',

    'desert': '''<defs><linearGradient id="dsky" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fcd34d"/><stop offset="100%" stop-color="#f59e0b"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#dsky)"/>
    <ellipse cx="32" cy="64" rx="48" ry="20" fill="#d97706"/>
    <ellipse cx="56" cy="56" rx="12" ry="8" fill="#b45309"/>
    <circle cx="12" cy="12" r="6" fill="#fef08a"/>''',

    'mountain': '''<defs><linearGradient id="msky" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#60a5fa"/><stop offset="100%" stop-color="#93c5fd"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#msky)"/>
    <polygon points="0,64 24,20 48,64" fill="#6b7280"/><polygon points="24,20 32,28 40,20" fill="white"/>
    <polygon points="32,64 48,32 64,64" fill="#9ca3af"/><polygon points="48,32 52,36 56,32" fill="white"/>''',

    'space': '''<rect width="64" height="64" fill="#030712"/>
    <circle cx="48" cy="32" r="12" fill="#7c3aed"/>
    <ellipse cx="48" cy="32" rx="18" ry="4" fill="none" stroke="#a78bfa" stroke-width="2" opacity="0.5"/>
    <circle cx="8" cy="12" r="1" fill="white"/><circle cx="24" cy="8" r="0.5" fill="white"/>
    <circle cx="56" cy="52" r="1" fill="white"/><circle cx="12" cy="48" r="0.5" fill="white"/>''',

    'galaxy': '''<defs><radialGradient id="gal" cx="50%" cy="50%">
    <stop offset="0%" stop-color="#c4b5fd"/><stop offset="50%" stop-color="#6d28d9"/>
    <stop offset="100%" stop-color="#1e1b4b"/></radialGradient></defs>
    <rect width="64" height="64" fill="#0f0a1a"/>
    <ellipse cx="32" cy="32" rx="28" ry="20" fill="url(#gal)" opacity="0.6" transform="rotate(-30 32 32)"/>
    <circle cx="32" cy="32" r="4" fill="#fef08a"/><circle cx="8" cy="8" r="0.5" fill="white"/>
    <circle cx="56" cy="12" r="1" fill="white"/><circle cx="12" cy="52" r="0.5" fill="white"/>''',

    'cave': '''<defs><radialGradient id="cav" cx="50%" cy="0%">
    <stop offset="0%" stop-color="#44403c"/><stop offset="100%" stop-color="#1c1917"/></radialGradient></defs>
    <rect width="64" height="64" fill="url(#cav)"/>
    <polygon points="0,0 12,16 8,0" fill="#292524"/><polygon points="20,0 28,12 36,0" fill="#292524"/>
    <polygon points="48,0 52,20 56,0" fill="#292524"/><polygon points="60,0 64,8 64,0" fill="#292524"/>
    <ellipse cx="32" cy="48" rx="20" ry="12" fill="#78716c" opacity="0.3"/>''',

    'castle': '''<defs><linearGradient id="cas" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fbbf24"/><stop offset="100%" stop-color="#7c3aed"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#cas)"/>
    <rect x="20" y="32" width="24" height="32" fill="#57534e"/>
    <rect x="16" y="24" width="8" height="40" fill="#44403c"/>
    <rect x="40" y="24" width="8" height="40" fill="#44403c"/>
    <rect x="18" y="20" width="4" height="8" fill="#44403c"/><rect x="42" y="20" width="4" height="8" fill="#44403c"/>
    <rect x="28" y="48" width="8" height="16" rx="4" fill="#78350f"/>''',

    'beach': '''<defs><linearGradient id="bsky" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#38bdf8"/><stop offset="100%" stop-color="#7dd3fc"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#bsky)"/>
    <rect x="0" y="40" width="64" height="24" fill="#0ea5e9"/>
    <path d="M0,40 Q16,36 32,40 T64,40" fill="#22d3ee"/>
    <ellipse cx="32" cy="64" rx="40" ry="12" fill="#fde68a"/>
    <circle cx="52" cy="12" r="8" fill="#fcd34d"/>''',

    'meadow': '''<defs><linearGradient id="msky" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#7dd3fc"/><stop offset="100%" stop-color="#bae6fd"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#msky)"/>
    <ellipse cx="32" cy="72" rx="48" ry="24" fill="#86efac"/>
    <circle cx="12" cy="48" r="3" fill="#f472b6"/><circle cx="28" cy="52" r="2" fill="#fbbf24"/>
    <circle cx="44" cy="50" r="3" fill="#a78bfa"/><circle cx="56" cy="54" r="2" fill="#f87171"/>''',

    'neon_city': '''<rect width="64" height="64" fill="#0f172a"/>
    <rect x="4" y="24" width="12" height="40" fill="#1e293b"/>
    <rect x="20" y="16" width="10" height="48" fill="#1e293b"/>
    <rect x="34" y="20" width="14" height="44" fill="#1e293b"/>
    <rect x="52" y="28" width="10" height="36" fill="#1e293b"/>
    <rect x="6" y="28" width="2" height="4" fill="#f472b6"/><rect x="10" y="32" width="2" height="4" fill="#22d3ee"/>
    <rect x="22" y="20" width="2" height="4" fill="#a78bfa"/><rect x="26" y="24" width="2" height="4" fill="#fbbf24"/>
    <rect x="38" y="24" width="2" height="4" fill="#22d3ee"/><rect x="42" y="28" width="2" height="4" fill="#f472b6"/>''',

    'underwater': '''<defs><linearGradient id="uw" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#0ea5e9"/><stop offset="100%" stop-color="#0c4a6e"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#uw)"/>
    <ellipse cx="8" cy="56" rx="6" ry="8" fill="#16a34a" opacity="0.7"/>
    <ellipse cx="56" cy="52" rx="8" ry="12" fill="#22c55e" opacity="0.7"/>
    <circle cx="12" cy="20" r="2" fill="white" opacity="0.3"/>
    <circle cx="32" cy="32" r="1.5" fill="white" opacity="0.3"/>
    <circle cx="48" cy="16" r="2" fill="white" opacity="0.3"/>''',

    'arctic': '''<defs><linearGradient id="arc" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#e0f2fe"/><stop offset="100%" stop-color="#bae6fd"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#arc)"/>
    <polygon points="0,40 20,32 40,44 64,36 64,64 0,64" fill="white"/>
    <polygon points="8,48 16,40 24,48" fill="#e0f2fe"/>
    <polygon points="40,52 52,44 60,54" fill="#e0f2fe"/>''',

    'autumn': '''<defs><linearGradient id="aut" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fed7aa"/><stop offset="100%" stop-color="#fdba74"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#aut)"/>
    <ellipse cx="32" cy="72" rx="48" ry="20" fill="#92400e"/>
    <polygon points="8,64 20,28 32,64" fill="#ea580c"/><polygon points="32,64 44,24 56,64" fill="#dc2626"/>
    <circle cx="12" cy="52" r="2" fill="#f97316"/><circle cx="48" cy="56" r="3" fill="#eab308"/>''',

    'candy': '''<rect width="64" height="64" fill="#fbcfe8"/>
    <circle cx="12" cy="12" r="8" fill="#f472b6"/><circle cx="52" cy="20" r="6" fill="#a78bfa"/>
    <circle cx="24" cy="44" r="10" fill="#22d3ee"/><circle cx="48" cy="52" r="8" fill="#fbbf24"/>
    <rect x="0" y="56" width="64" height="8" fill="#f9a8d4"/>
    <circle cx="8" cy="32" r="4" fill="#a3e635"/>''',

    'cherry_blossom': '''<defs><linearGradient id="cb" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#fce7f3"/><stop offset="100%" stop-color="#fbcfe8"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#cb)"/>
    <line x1="32" y1="64" x2="32" y2="20" stroke="#78350f" stroke-width="4"/>
    <line x1="32" y1="32" x2="16" y2="20" stroke="#78350f" stroke-width="2"/>
    <line x1="32" y1="36" x2="48" y2="24" stroke="#78350f" stroke-width="2"/>
    <g fill="#f9a8d4" opacity="0.8">
      <circle cx="32" cy="16" r="6"/><circle cx="16" cy="16" r="5"/><circle cx="48" cy="20" r="5"/>
      <circle cx="24" cy="8" r="4"/><circle cx="40" cy="12" r="4"/>
    </g>''',

    'cozy_fireplace': '''<rect width="64" height="64" fill="#292524"/>
    <rect x="16" y="32" width="32" height="32" fill="#78350f"/>
    <rect x="12" y="28" width="40" height="6" fill="#92400e"/>
    <rect x="20" y="40" width="24" height="24" fill="#1c1917"/>
    <ellipse cx="32" cy="56" rx="10" ry="12" fill="#f97316"/>
    <ellipse cx="28" cy="52" rx="4" ry="8" fill="#fbbf24"/>
    <ellipse cx="36" cy="54" rx="3" ry="6" fill="#fcd34d"/>''',

    'crystal_cave': '''<defs><linearGradient id="cc" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#1e1b4b"/><stop offset="100%" stop-color="#312e81"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#cc)"/>
    <polygon points="8,64 16,32 24,64" fill="#a78bfa" opacity="0.6"/>
    <polygon points="24,64 36,20 48,64" fill="#c4b5fd" opacity="0.7"/>
    <polygon points="40,64 52,28 64,64" fill="#8b5cf6" opacity="0.6"/>
    <polygon points="0,0 8,16 4,0" fill="#6d28d9" opacity="0.5"/>
    <polygon points="56,0 60,20 64,0" fill="#6d28d9" opacity="0.5"/>''',

    'enchanted': '''<defs><radialGradient id="ench">
    <stop offset="0%" stop-color="#c4b5fd"/><stop offset="100%" stop-color="#3b0764"/></radialGradient>
    <filter id="glow"><feGaussianBlur stdDeviation="2"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter></defs>
    <rect width="64" height="64" fill="url(#ench)"/>
    <circle cx="32" cy="32" r="8" fill="#fef08a" filter="url(#glow)" opacity="0.6"/>
    <circle cx="12" cy="16" r="1" fill="white"/><circle cx="52" cy="12" r="1.5" fill="white"/>
    <circle cx="8" cy="48" r="1" fill="white"/><circle cx="56" cy="52" r="1" fill="white"/>''',

    'jungle': '''<defs><linearGradient id="jun" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#166534"/><stop offset="100%" stop-color="#14532d"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#jun)"/>
    <ellipse cx="8" cy="32" rx="12" ry="32" fill="#22c55e" opacity="0.8"/>
    <ellipse cx="56" cy="36" rx="14" ry="28" fill="#16a34a" opacity="0.8"/>
    <ellipse cx="32" cy="40" rx="20" ry="24" fill="#15803d" opacity="0.7"/>
    <path d="M20,64 Q28,48 32,64" fill="#84cc16"/>
    <path d="M40,64 Q48,44 52,64" fill="#65a30d"/>''',

    'mystical_portal': '''<defs><radialGradient id="port">
    <stop offset="0%" stop-color="#fef08a"/><stop offset="50%" stop-color="#8b5cf6"/>
    <stop offset="100%" stop-color="#1e1b4b"/></radialGradient>
    <filter id="glow"><feGaussianBlur stdDeviation="3"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter></defs>
    <rect width="64" height="64" fill="#1e1b4b"/>
    <ellipse cx="32" cy="32" rx="20" ry="24" fill="url(#port)" filter="url(#glow)"/>
    <ellipse cx="32" cy="32" rx="12" ry="16" fill="#c4b5fd" opacity="0.3"/>''',

    'rainy': '''<defs><linearGradient id="rain" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#475569"/><stop offset="100%" stop-color="#334155"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#rain)"/>
    <g stroke="#94a3b8" stroke-width="1" opacity="0.5">
      <line x1="8" y1="0" x2="4" y2="16"/><line x1="20" y1="4" x2="16" y2="20"/>
      <line x1="32" y1="0" x2="28" y2="16"/><line x1="44" y1="8" x2="40" y2="24"/>
      <line x1="56" y1="0" x2="52" y2="16"/><line x1="12" y1="24" x2="8" y2="40"/>
      <line x1="28" y1="28" x2="24" y2="44"/><line x1="48" y1="24" x2="44" y2="40"/>
      <line x1="16" y1="44" x2="12" y2="60"/><line x1="36" y1="48" x2="32" y2="64"/>
      <line x1="52" y1="44" x2="48" y2="60"/>
    </g>''',

    'spring_meadow': '''<defs><linearGradient id="spr" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#7dd3fc"/><stop offset="100%" stop-color="#a5f3fc"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#spr)"/>
    <ellipse cx="32" cy="72" rx="48" ry="20" fill="#4ade80"/>
    <circle cx="8" cy="48" r="2" fill="#fbbf24"/><circle cx="16" cy="52" r="3" fill="#f472b6"/>
    <circle cx="28" cy="48" r="2" fill="#a78bfa"/><circle cx="40" cy="54" r="3" fill="#fbbf24"/>
    <circle cx="52" cy="50" r="2" fill="#f472b6"/><circle cx="60" cy="56" r="2" fill="#a78bfa"/>
    <ellipse cx="16" cy="16" rx="8" ry="4" fill="white" opacity="0.8"/>''',

    'volcanic': '''<defs><linearGradient id="vol" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0%" stop-color="#7f1d1d"/><stop offset="50%" stop-color="#450a0a"/>
    <stop offset="100%" stop-color="#1c1917"/></linearGradient></defs>
    <rect width="64" height="64" fill="url(#vol)"/>
    <polygon points="16,64 32,20 48,64" fill="#292524"/>
    <ellipse cx="32" cy="24" rx="6" ry="4" fill="#f97316"/>
    <ellipse cx="32" cy="24" rx="3" ry="2" fill="#fcd34d"/>
    <path d="M28,28 Q24,40 20,64" fill="#f97316" opacity="0.6"/>
    <path d="M36,28 Q40,44 44,64" fill="#dc2626" opacity="0.6"/>'''
}

def generate_backgrounds():
    base = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'backgrounds')
    os.makedirs(base, exist_ok=True)

    for name, content in BACKGROUNDS.items():
        svg = f'{svg_header()}\n{content}\n{svg_footer()}'
        filepath = os.path.join(base, f'{name}.svg')
        with open(filepath, 'w') as f:
            f.write(svg)
        print(f'Created backgrounds/{name}.svg')

if __name__ == '__main__':
    generate_backgrounds()
    print('\nBackgrounds generated!')
