#!/usr/bin/env python3
"""Generate aura effect SVG assets for exambuilder"""

import os

def svg_header(w=64, h=64):
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">'

def svg_footer():
    return '</svg>'

AURAS = {
    'sparkles': '''<defs>
    <filter id="glow"><feGaussianBlur stdDeviation="1"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter>
    </defs>
    <g filter="url(#glow)">
      <circle cx="12" cy="16" r="2" fill="#fef08a"><animate attributeName="opacity" values="0;1;0" dur="1.5s" begin="0s" repeatCount="indefinite"/><animate attributeName="r" values="1;3;1" dur="1.5s" begin="0s" repeatCount="indefinite"/></circle>
      <circle cx="52" cy="12" r="2" fill="#fef08a"><animate attributeName="opacity" values="0;1;0" dur="1.5s" begin="0.3s" repeatCount="indefinite"/><animate attributeName="r" values="1;2;1" dur="1.5s" begin="0.3s" repeatCount="indefinite"/></circle>
      <circle cx="8" cy="44" r="2" fill="#fef08a"><animate attributeName="opacity" values="0;1;0" dur="1.5s" begin="0.6s" repeatCount="indefinite"/><animate attributeName="r" values="1;2.5;1" dur="1.5s" begin="0.6s" repeatCount="indefinite"/></circle>
      <circle cx="56" cy="48" r="2" fill="#fef08a"><animate attributeName="opacity" values="0;1;0" dur="1.5s" begin="0.9s" repeatCount="indefinite"/><animate attributeName="r" values="1;2;1" dur="1.5s" begin="0.9s" repeatCount="indefinite"/></circle>
      <circle cx="32" cy="8" r="2" fill="#fef08a"><animate attributeName="opacity" values="0;1;0" dur="1.5s" begin="1.2s" repeatCount="indefinite"/><animate attributeName="r" values="1;3;1" dur="1.5s" begin="1.2s" repeatCount="indefinite"/></circle>
      <polygon points="20,56 22,52 24,56 20,54" fill="#fef08a"><animate attributeName="opacity" values="0;1;0" dur="2s" begin="0.4s" repeatCount="indefinite"/></polygon>
      <polygon points="44,58 46,54 48,58 44,56" fill="#fef08a"><animate attributeName="opacity" values="0;1;0" dur="2s" begin="1s" repeatCount="indefinite"/></polygon>
    </g>''',

    'hearts': '''<defs>
    <filter id="glow"><feGaussianBlur stdDeviation="1"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter>
    </defs>
    <g filter="url(#glow)">
      <path d="M12,20 Q12,16 14,16 Q16,16 16,18 Q16,16 18,16 Q20,16 20,20 Q20,24 16,26 Q12,24 12,20 Z" fill="#f472b6"><animate attributeName="opacity" values="0;1;0" dur="2s" begin="0s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="translate" values="0,0;0,-8" dur="2s" begin="0s" repeatCount="indefinite"/></path>
      <path d="M48,16 Q48,12 50,12 Q52,12 52,14 Q52,12 54,12 Q56,12 56,16 Q56,20 52,22 Q48,20 48,16 Z" fill="#f472b6"><animate attributeName="opacity" values="0;1;0" dur="2s" begin="0.5s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="translate" values="0,0;0,-10" dur="2s" begin="0.5s" repeatCount="indefinite"/></path>
      <path d="M6,44 Q6,40 8,40 Q10,40 10,42 Q10,40 12,40 Q14,40 14,44 Q14,48 10,50 Q6,48 6,44 Z" fill="#f472b6"><animate attributeName="opacity" values="0;1;0" dur="2s" begin="1s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="translate" values="0,0;0,-6" dur="2s" begin="1s" repeatCount="indefinite"/></path>
      <path d="M50,52 Q50,48 52,48 Q54,48 54,50 Q54,48 56,48 Q58,48 58,52 Q58,56 54,58 Q50,56 50,52 Z" fill="#f472b6"><animate attributeName="opacity" values="0;1;0" dur="2s" begin="1.5s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="translate" values="0,0;0,-8" dur="2s" begin="1.5s" repeatCount="indefinite"/></path>
    </g>''',

    'stars': '''<defs>
    <filter id="glow"><feGaussianBlur stdDeviation="1.5"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter>
    </defs>
    <g filter="url(#glow)">
      <polygon points="10,18 12,14 14,18 10,16 14,16" fill="#a78bfa"><animate attributeName="opacity" values="0.3;1;0.3" dur="1s" begin="0s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="rotate" values="0 12 16;180 12 16;360 12 16" dur="3s" repeatCount="indefinite"/></polygon>
      <polygon points="50,12 52,8 54,12 50,10 54,10" fill="#a78bfa"><animate attributeName="opacity" values="0.3;1;0.3" dur="1s" begin="0.3s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="rotate" values="0 52 10;180 52 10;360 52 10" dur="3s" repeatCount="indefinite"/></polygon>
      <polygon points="6,48 8,44 10,48 6,46 10,46" fill="#c4b5fd"><animate attributeName="opacity" values="0.3;1;0.3" dur="1s" begin="0.6s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="rotate" values="0 8 46;180 8 46;360 8 46" dur="3s" repeatCount="indefinite"/></polygon>
      <polygon points="54,54 56,50 58,54 54,52 58,52" fill="#c4b5fd"><animate attributeName="opacity" values="0.3;1;0.3" dur="1s" begin="0.9s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="rotate" values="0 56 52;180 56 52;360 56 52" dur="3s" repeatCount="indefinite"/></polygon>
      <polygon points="30,6 32,2 34,6 30,4 34,4" fill="#a78bfa"><animate attributeName="opacity" values="0.3;1;0.3" dur="1s" begin="0.5s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="rotate" values="0 32 4;180 32 4;360 32 4" dur="3s" repeatCount="indefinite"/></polygon>
    </g>''',

    'flames': '''<defs>
    <linearGradient id="flame" x1="0" y1="1" x2="0" y2="0">
      <stop offset="0%" stop-color="#f97316"/><stop offset="50%" stop-color="#fbbf24"/><stop offset="100%" stop-color="#fef08a"/>
    </linearGradient>
    <filter id="blur"><feGaussianBlur stdDeviation="1"/></filter>
    </defs>
    <g filter="url(#blur)">
      <ellipse cx="10" cy="56" rx="4" ry="8" fill="url(#flame)"><animate attributeName="ry" values="8;12;8" dur="0.3s" repeatCount="indefinite"/></ellipse>
      <ellipse cx="20" cy="58" rx="3" ry="6" fill="url(#flame)"><animate attributeName="ry" values="6;10;6" dur="0.25s" repeatCount="indefinite"/></ellipse>
      <ellipse cx="32" cy="56" rx="5" ry="10" fill="url(#flame)"><animate attributeName="ry" values="10;14;10" dur="0.35s" repeatCount="indefinite"/></ellipse>
      <ellipse cx="44" cy="58" rx="3" ry="6" fill="url(#flame)"><animate attributeName="ry" values="6;9;6" dur="0.28s" repeatCount="indefinite"/></ellipse>
      <ellipse cx="54" cy="56" rx="4" ry="8" fill="url(#flame)"><animate attributeName="ry" values="8;11;8" dur="0.32s" repeatCount="indefinite"/></ellipse>
    </g>''',

    'rainbow': '''<defs>
    <linearGradient id="rain" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#ef4444"/><stop offset="17%" stop-color="#f97316"/>
      <stop offset="33%" stop-color="#fbbf24"/><stop offset="50%" stop-color="#22c55e"/>
      <stop offset="67%" stop-color="#3b82f6"/><stop offset="83%" stop-color="#8b5cf6"/>
      <stop offset="100%" stop-color="#ec4899"/>
    </linearGradient>
    </defs>
    <path d="M4,60 Q4,20 32,8 Q60,20 60,60" fill="none" stroke="url(#rain)" stroke-width="3" opacity="0.6">
      <animate attributeName="stroke-width" values="3;5;3" dur="2s" repeatCount="indefinite"/>
    </path>
    <path d="M10,60 Q10,28 32,16 Q54,28 54,60" fill="none" stroke="url(#rain)" stroke-width="2" opacity="0.4">
      <animate attributeName="stroke-width" values="2;4;2" dur="2s" begin="0.5s" repeatCount="indefinite"/>
    </path>''',

    'bubbles': '''<defs>
    <radialGradient id="bub" cx="30%" cy="30%">
      <stop offset="0%" stop-color="white" stop-opacity="0.8"/>
      <stop offset="100%" stop-color="#60a5fa" stop-opacity="0.2"/>
    </radialGradient>
    </defs>
    <circle cx="12" cy="48" r="4" fill="url(#bub)" stroke="#93c5fd" stroke-width="0.5"><animate attributeName="cy" values="48;8" dur="3s" repeatCount="indefinite"/><animate attributeName="r" values="4;6;4" dur="3s" repeatCount="indefinite"/><animate attributeName="opacity" values="0.8;0" dur="3s" repeatCount="indefinite"/></circle>
    <circle cx="28" cy="56" r="3" fill="url(#bub)" stroke="#93c5fd" stroke-width="0.5"><animate attributeName="cy" values="56;12" dur="3.5s" begin="0.5s" repeatCount="indefinite"/><animate attributeName="opacity" values="0.8;0" dur="3.5s" begin="0.5s" repeatCount="indefinite"/></circle>
    <circle cx="44" cy="52" r="5" fill="url(#bub)" stroke="#93c5fd" stroke-width="0.5"><animate attributeName="cy" values="52;4" dur="4s" begin="1s" repeatCount="indefinite"/><animate attributeName="r" values="5;7;5" dur="4s" begin="1s" repeatCount="indefinite"/><animate attributeName="opacity" values="0.8;0" dur="4s" begin="1s" repeatCount="indefinite"/></circle>
    <circle cx="52" cy="58" r="2" fill="url(#bub)" stroke="#93c5fd" stroke-width="0.5"><animate attributeName="cy" values="58;16" dur="2.5s" begin="1.5s" repeatCount="indefinite"/><animate attributeName="opacity" values="0.8;0" dur="2.5s" begin="1.5s" repeatCount="indefinite"/></circle>''',

    'leaves': '''<defs>
    <linearGradient id="leaf" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#86efac"/><stop offset="100%" stop-color="#22c55e"/>
    </linearGradient>
    </defs>
    <g>
      <path d="M8,8 Q12,4 16,8 Q12,12 8,8 Z" fill="url(#leaf)"><animateMotion path="M0,0 C10,20 -5,40 10,56" dur="4s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="rotate" values="0;360" dur="4s" repeatCount="indefinite"/></path>
      <path d="M48,4 Q52,0 56,4 Q52,8 48,4 Z" fill="url(#leaf)"><animateMotion path="M0,0 C-8,24 8,36 -4,60" dur="5s" begin="1s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="rotate" values="0;-360" dur="5s" begin="1s" repeatCount="indefinite"/></path>
      <path d="M28,2 Q32,-2 36,2 Q32,6 28,2 Z" fill="url(#leaf)"><animateMotion path="M0,0 C12,16 -8,32 6,62" dur="4.5s" begin="2s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="rotate" values="0;360" dur="4.5s" begin="2s" repeatCount="indefinite"/></path>
    </g>''',

    'snow': '''<defs>
    <filter id="blur"><feGaussianBlur stdDeviation="0.5"/></filter>
    </defs>
    <g filter="url(#blur)">
      <circle cx="10" cy="4" r="2" fill="white"><animate attributeName="cy" values="4;64" dur="4s" repeatCount="indefinite"/><animate attributeName="cx" values="10;14;10" dur="2s" repeatCount="indefinite"/></circle>
      <circle cx="24" cy="8" r="1.5" fill="white"><animate attributeName="cy" values="8;68" dur="3.5s" begin="0.5s" repeatCount="indefinite"/><animate attributeName="cx" values="24;20;24" dur="1.8s" repeatCount="indefinite"/></circle>
      <circle cx="38" cy="2" r="2" fill="white"><animate attributeName="cy" values="2;62" dur="4.2s" begin="1s" repeatCount="indefinite"/><animate attributeName="cx" values="38;42;38" dur="2.2s" repeatCount="indefinite"/></circle>
      <circle cx="52" cy="6" r="1" fill="white"><animate attributeName="cy" values="6;66" dur="3s" begin="1.5s" repeatCount="indefinite"/><animate attributeName="cx" values="52;48;52" dur="1.5s" repeatCount="indefinite"/></circle>
      <circle cx="16" cy="12" r="1.5" fill="white"><animate attributeName="cy" values="12;72" dur="3.8s" begin="2s" repeatCount="indefinite"/><animate attributeName="cx" values="16;20;16" dur="1.9s" repeatCount="indefinite"/></circle>
      <circle cx="46" cy="10" r="2" fill="white"><animate attributeName="cy" values="10;70" dur="4.5s" begin="0.8s" repeatCount="indefinite"/><animate attributeName="cx" values="46;42;46" dur="2.1s" repeatCount="indefinite"/></circle>
    </g>''',

    'lightning': '''<defs>
    <filter id="elec"><feGaussianBlur stdDeviation="2"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter>
    </defs>
    <g filter="url(#elec)">
      <path d="M12,8 L8,24 L14,24 L10,40" stroke="#60a5fa" stroke-width="2" fill="none"><animate attributeName="opacity" values="0;1;0" dur="0.2s" begin="0s;0.8s;1.6s;2.4s" repeatCount="indefinite"/></path>
      <path d="M52,4 L48,20 L54,20 L50,36" stroke="#60a5fa" stroke-width="2" fill="none"><animate attributeName="opacity" values="0;1;0" dur="0.15s" begin="0.3s;1.1s;1.9s;2.7s" repeatCount="indefinite"/></path>
      <path d="M32,2 L28,14 L34,14 L30,28" stroke="#93c5fd" stroke-width="3" fill="none"><animate attributeName="opacity" values="0;1;0" dur="0.25s" begin="0.5s;1.3s;2.1s;2.9s" repeatCount="indefinite"/></path>
    </g>''',

    'music': '''<defs>
    <filter id="glow"><feGaussianBlur stdDeviation="1"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter>
    </defs>
    <g filter="url(#glow)">
      <g fill="#a78bfa"><ellipse cx="10" cy="48" rx="4" ry="3"/><rect x="12" y="32" width="2" height="16"/><path d="M14,32 Q20,28 20,36" fill="none" stroke="#a78bfa" stroke-width="2"/>
        <animateTransform attributeName="transform" type="translate" values="0,0;0,-16" dur="2s" repeatCount="indefinite"/>
        <animate attributeName="opacity" values="0;1;0" dur="2s" repeatCount="indefinite"/>
      </g>
      <g fill="#c4b5fd"><ellipse cx="50" cy="44" rx="3" ry="2"/><rect x="51" y="30" width="2" height="14"/><path d="M53,30 Q58,27 58,34" fill="none" stroke="#c4b5fd" stroke-width="2"/>
        <animateTransform attributeName="transform" type="translate" values="0,0;0,-20" dur="2.5s" begin="0.5s" repeatCount="indefinite"/>
        <animate attributeName="opacity" values="0;1;0" dur="2.5s" begin="0.5s" repeatCount="indefinite"/>
      </g>
      <g fill="#8b5cf6"><ellipse cx="28" cy="52" rx="4" ry="3"/><rect x="30" y="36" width="2" height="16"/>
        <animateTransform attributeName="transform" type="translate" values="0,0;0,-18" dur="2.2s" begin="1s" repeatCount="indefinite"/>
        <animate attributeName="opacity" values="0;1;0" dur="2.2s" begin="1s" repeatCount="indefinite"/>
      </g>
    </g>''',

    'coins': '''<defs>
    <linearGradient id="coin" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#fcd34d"/><stop offset="100%" stop-color="#b45309"/>
    </linearGradient>
    <filter id="glow"><feGaussianBlur stdDeviation="1"/><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter>
    </defs>
    <g filter="url(#glow)">
      <ellipse cx="12" cy="48" rx="6" ry="4" fill="url(#coin)"><animate attributeName="cy" values="48;8" dur="2s" repeatCount="indefinite"/><animate attributeName="opacity" values="1;0" dur="2s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="rotate" values="0 12 48;360 12 8" dur="2s" repeatCount="indefinite"/></ellipse>
      <ellipse cx="32" cy="52" rx="6" ry="4" fill="url(#coin)"><animate attributeName="cy" values="52;4" dur="2.5s" begin="0.5s" repeatCount="indefinite"/><animate attributeName="opacity" values="1;0" dur="2.5s" begin="0.5s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="rotate" values="0 32 52;360 32 4" dur="2.5s" begin="0.5s" repeatCount="indefinite"/></ellipse>
      <ellipse cx="52" cy="50" rx="6" ry="4" fill="url(#coin)"><animate attributeName="cy" values="50;6" dur="2.2s" begin="1s" repeatCount="indefinite"/><animate attributeName="opacity" values="1;0" dur="2.2s" begin="1s" repeatCount="indefinite"/><animateTransform attributeName="transform" type="rotate" values="0 52 50;360 52 6" dur="2.2s" begin="1s" repeatCount="indefinite"/></ellipse>
    </g>''',

    'magic': '''<defs>
    <filter id="glow"><feGaussianBlur stdDeviation="2"/><feMerge><feMergeNode/><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter>
    <linearGradient id="mag" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#c4b5fd"/><stop offset="50%" stop-color="#8b5cf6"/><stop offset="100%" stop-color="#6d28d9"/>
    </linearGradient>
    </defs>
    <g filter="url(#glow)">
      <circle cx="32" cy="32" r="24" fill="none" stroke="url(#mag)" stroke-width="2" stroke-dasharray="8 4"><animateTransform attributeName="transform" type="rotate" values="0 32 32;360 32 32" dur="4s" repeatCount="indefinite"/></circle>
      <circle cx="32" cy="32" r="16" fill="none" stroke="url(#mag)" stroke-width="1.5" stroke-dasharray="6 3"><animateTransform attributeName="transform" type="rotate" values="360 32 32;0 32 32" dur="3s" repeatCount="indefinite"/></circle>
      <polygon points="32,12 34,28 50,28 38,36 42,52 32,42 22,52 26,36 14,28 30,28" fill="#a78bfa" opacity="0.3"><animate attributeName="opacity" values="0.2;0.5;0.2" dur="2s" repeatCount="indefinite"/></polygon>
    </g>'''
}

def generate_auras():
    base = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'auras')
    os.makedirs(base, exist_ok=True)

    for name, content in AURAS.items():
        svg = f'{svg_header()}\n{content}\n{svg_footer()}'
        filepath = os.path.join(base, f'{name}.svg')
        with open(filepath, 'w') as f:
            f.write(svg)
        print(f'Created auras/{name}.svg')

if __name__ == '__main__':
    generate_auras()
    print('\nAuras generated!')
