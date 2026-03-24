#!/usr/bin/env python3
"""
Master script to generate all vector art assets for exambuilder.
Run this script to regenerate all SVG sprites.
"""

import os
import sys

# Add current directory to path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from generate_vectors import generate_creatures
from generate_hats import generate_hats
from generate_held import generate_held
from generate_auras import generate_auras
from generate_backgrounds import generate_backgrounds

def main():
    print("=" * 50)
    print("ExamBuilder Vector Asset Generator")
    print("=" * 50)
    print()

    print("Generating creature avatars...")
    generate_creatures()
    print()

    print("Generating hats...")
    generate_hats()
    print()

    print("Generating held items...")
    generate_held()
    print()

    print("Generating aura effects...")
    generate_auras()
    print()

    print("Generating backgrounds...")
    generate_backgrounds()
    print()

    print("=" * 50)
    print("All assets generated successfully!")
    print("=" * 50)

    # Summary
    base = os.path.dirname(os.path.abspath(__file__))
    counts = {
        'creatures': sum(1 for d in ['cat', 'slime', 'octopus', 'snail']
                        for f in os.listdir(os.path.join(base, d))
                        if f.endswith('.svg')),
        'hats': len([f for f in os.listdir(os.path.join(base, 'hats')) if f.endswith('.svg')]),
        'held': len([f for f in os.listdir(os.path.join(base, 'held')) if f.endswith('.svg')]),
        'auras': len([f for f in os.listdir(os.path.join(base, 'auras')) if f.endswith('.svg')]),
        'backgrounds': len([f for f in os.listdir(os.path.join(base, 'backgrounds')) if f.endswith('.svg')]),
    }

    print()
    print("Summary:")
    for category, count in counts.items():
        print(f"  {category}: {count} SVGs")
    print(f"  Total: {sum(counts.values())} SVGs")

if __name__ == '__main__':
    main()
