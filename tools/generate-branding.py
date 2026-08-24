#!/usr/bin/env python3
"""Generate original RF-5 branding assets required by RackForge."""

from __future__ import annotations

import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "plugin" / "package" / "branding"
FONT_CANDIDATES = (
    Path("C:/Windows/Fonts/arialbd.ttf"),
    Path("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"),
)

PANEL = (28, 28, 29)
PANEL_LIGHT = (58, 57, 56)
INK = (242, 239, 226)
MUTED = (180, 176, 164)
COPPER = (195, 58, 44)
AMBER = (239, 167, 49)
WOOD_DARK = (49, 25, 18)
WOOD_LIGHT = (112, 60, 37)


def font(size: int) -> ImageFont.FreeTypeFont:
    for candidate in FONT_CANDIDATES:
        if candidate.is_file():
            return ImageFont.truetype(str(candidate), size)
    return ImageFont.load_default(size=size)


def panel(width: int, height: int) -> Image.Image:
    image = Image.new("RGB", (width, height), PANEL)
    pixels = image.load()
    center = width * 0.44
    sigma = width * 0.42
    for x in range(width):
        highlight = math.exp(-((x - center) ** 2) / (2.0 * sigma * sigma))
        for y in range(height):
            grain = 0.018 * math.sin(x * 0.17 + y * 0.013)
            mix = max(0.0, min(1.0, highlight * 0.34 + grain))
            pixels[x, y] = tuple(
                round(PANEL[channel] + (PANEL_LIGHT[channel] - PANEL[channel]) * mix)
                for channel in range(3)
            )
    return image


def wood(image: Image.Image, box: tuple[int, int, int, int]) -> None:
    left, top, right, bottom = box
    draw = ImageDraw.Draw(image, "RGBA")
    draw.rectangle(box, fill=WOOD_DARK)
    for offset in range(left, right, 7):
        wave = 4 * math.sin(offset * 0.11)
        draw.line(
            (offset, top, offset + wave, bottom),
            fill=(*WOOD_LIGHT, 80),
            width=2,
        )


def centered(draw: ImageDraw.ImageDraw, box: tuple[int, int, int, int], text: str, size: int, fill=INK) -> None:
    face = font(size)
    bounds = draw.textbbox((0, 0), text, font=face)
    width = bounds[2] - bounds[0]
    height = bounds[3] - bounds[1]
    x = box[0] + (box[2] - box[0] - width) / 2
    y = box[1] + (box[3] - box[1] - height) / 2 - bounds[1]
    draw.text((x, y), text, font=face, fill=fill)


def glow_dot(image: Image.Image, center: tuple[int, int], radius: int) -> None:
    x, y = center
    glow = Image.new("RGBA", image.size, (0, 0, 0, 0))
    glow_draw = ImageDraw.Draw(glow)
    glow_draw.ellipse((x - radius * 4, y - radius * 4, x + radius * 4, y + radius * 4), fill=(*AMBER, 70))
    glow = glow.filter(ImageFilter.GaussianBlur(radius * 2.0))
    image.paste(glow, (0, 0), glow)
    draw = ImageDraw.Draw(image)
    draw.ellipse((x - radius, y - radius, x + radius, y + radius), fill=AMBER)


def oscillator_lines(draw: ImageDraw.ImageDraw, box: tuple[int, int, int, int], width: int) -> None:
    left, top, right, bottom = box
    middle = (top + bottom) / 2
    points_a = []
    points_b = []
    for x in range(left, right + 1, 3):
        phase = (x - left) / max(1, right - left)
        points_a.append((x, middle - 24 + 18 * math.sin(phase * math.tau * 3)))
        saw = ((phase * 4.0) % 1.0) * 2.0 - 1.0
        points_b.append((x, middle + 24 + saw * 18))
    draw.line(points_a, fill=INK, width=width, joint="curve")
    draw.line(points_b, fill=COPPER, width=width, joint="curve")


def identity(image: Image.Image, box: tuple[int, int, int, int], title_size: int, subtitle_size: int) -> None:
    draw = ImageDraw.Draw(image)
    left, top, right, bottom = box
    centered(draw, (left, top, right, top + int((bottom - top) * 0.66)), "RF-5", title_size)
    centered(
        draw,
        (left, top + int((bottom - top) * 0.62), right, bottom),
        "FIVE-VOICE POLYPHONIC SYNTHESIZER",
        subtitle_size,
        MUTED,
    )


def make_icon() -> None:
    image = panel(512, 512)
    wood(image, (0, 0, 48, 512))
    wood(image, (464, 0, 512, 512))
    draw = ImageDraw.Draw(image)
    draw.rectangle((48, 44, 464, 59), fill=COPPER)
    identity(image, (64, 78, 448, 235), 90, 17)
    oscillator_lines(draw, (100, 245, 412, 350), 4)
    for index in range(5):
        glow_dot(image, (156 + index * 50, 409), 6)
    image.save(OUTPUT / "icon.png", optimize=True)


def make_banner() -> None:
    image = panel(1600, 400)
    wood(image, (0, 0, 90, 400))
    wood(image, (1510, 0, 1600, 400))
    draw = ImageDraw.Draw(image)
    draw.rectangle((90, 30, 1510, 50), fill=COPPER)
    identity(image, (130, 80, 785, 300), 132, 24)
    oscillator_lines(draw, (860, 90, 1450, 250), 5)
    for index in range(5):
        glow_dot(image, (1035 + index * 78, 316), 7)
    image.save(OUTPUT / "banner.png", optimize=True)


def make_splash() -> None:
    image = panel(1920, 1080)
    wood(image, (0, 0, 145, 1080))
    wood(image, (1775, 0, 1920, 1080))
    draw = ImageDraw.Draw(image)
    draw.rectangle((145, 96, 1775, 124), fill=COPPER)
    identity(image, (240, 190, 1680, 520), 210, 36)
    oscillator_lines(draw, (355, 535, 1565, 765), 7)
    draw.line((355, 830, 1565, 830), fill=(*INK, 150), width=3)
    for index in range(5):
        glow_dot(image, (660 + index * 150, 890), 10)
    centered(draw, (350, 940, 1570, 1010), "RUST / WEBASSEMBLY / RACKFORGE", 27, MUTED)
    image.save(OUTPUT / "splash.png", optimize=True)


def main() -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    make_icon()
    make_banner()
    make_splash()
    for path in sorted(OUTPUT.glob("*.png")):
        with Image.open(path) as image:
            print(f"BRANDING_WRITTEN path={path} size={image.width}x{image.height} bytes={path.stat().st_size}")


if __name__ == "__main__":
    main()
