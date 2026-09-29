#!/usr/bin/env python3
"""Build the RF-5 case's walnut from its raw photograph.

The varnish is baked into the texture, not layered over it in CSS: a warm
clear coat multiplies the raw walnut by a tint at an opacity, which deepens
and saturates it. Two files come out:

- `walnut-satin.jpg`, the full-resolution walnut for the rails;
- `walnut-satin-tall.jpg`, half the resolution and followed by its mirror
  image, so it repeats down a case of any height without a seam.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageOps


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets" / "branding-source" / "walnut-satin-raw.jpg"
OUTPUT = ROOT / "plugin" / "package" / "web" / "assets"

VARNISH_TINT = (150, 60, 18)
VARNISH_OPACITY = 0.36


def varnish(image: Image.Image) -> Image.Image:
    """Multiply by the tint at the varnish's opacity, channel by channel."""
    bands = [
        band.point(lambda value, tint=tint: round(value * (1 - VARNISH_OPACITY * (1 - tint / 255))))
        for band, tint in zip(image.convert("RGB").split(), VARNISH_TINT)
    ]
    return Image.merge("RGB", bands)


def main() -> None:
    walnut = varnish(Image.open(SOURCE))
    walnut.save(OUTPUT / "walnut-satin.jpg", quality=88, optimize=True, progressive=True)

    half = walnut.resize((walnut.width // 2, walnut.height // 2), Image.LANCZOS)
    tall = Image.new("RGB", (half.width, half.height * 2))
    tall.paste(half, (0, 0))
    tall.paste(ImageOps.flip(half), (0, half.height))
    tall.save(OUTPUT / "walnut-satin-tall.jpg", quality=86, optimize=True, progressive=True)


if __name__ == "__main__":
    main()
