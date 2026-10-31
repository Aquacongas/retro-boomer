#!/usr/bin/env python3
"""Validate the official, supplied Retro Boomer cassette icons without modifying them.

The legacy command name is retained for existing local workflows. The former
SVG/P-mark generator is retired: official cassette artwork is supplied ready-made,
so this command never generates, recolors or overwrites any image.
"""
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'noid_gui/assets/app-icons'
SIZES = (16, 24, 32, 48, 64, 128, 256, 512, 1024)


def main():
    for size in SIZES:
        path = OUTPUT / f'retro-boomer-{size}.png'
        with Image.open(path) as image:
            if image.format != 'PNG' or image.size != (size, size):
                raise ValueError(f'Invalid icon format or dimensions: {path.name}')
            if image.mode != 'RGBA' or image.getchannel('A').getextrema() != (0, 255):
                raise ValueError(f'Expected transparent RGBA artwork: {path.name}')
    with Image.open(OUTPUT / 'retro-boomer.ico') as icon:
        expected = {(n, n) for n in SIZES if n <= 256}
        if not expected.issubset(icon.ico.sizes()):
            raise ValueError('retro-boomer.ico is missing a required Windows icon size')
        for size in expected:
            icon.ico.getimage(size).load()
    print('Official Retro Boomer PNG/ICO icons verified; no images generated or modified.')


if __name__ == '__main__':
    main()
