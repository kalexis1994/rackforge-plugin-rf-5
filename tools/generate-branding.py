#!/usr/bin/env python3
"""Build the canonical RF-5 product-photography branding assets.

The committed masters are deliberately lossy JPEG files: RackForge packages
only the final, fully validated RGB PNGs, while contributors retain enough
resolution to regenerate the three canonical crops without keeping large
generation outputs in the repository.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageOps


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets" / "branding-source"
OUTPUT = ROOT / "plugin" / "package" / "branding"


@dataclass(frozen=True)
class Asset:
    source: str
    output: str
    width: int
    height: int
    center_x: float = 0.5
    center_y: float = 0.5
    maximum_bytes: int = 0
    preserve_alpha: bool = False


ASSETS = (
    Asset(
        "icon-master.png",
        "icon.png",
        512,
        512,
        maximum_bytes=2 * 1024 * 1024,
        preserve_alpha=True,
    ),
    Asset(
        "banner-master.jpg",
        "banner.png",
        1600,
        400,
        center_x=0.5,
        center_y=0.53,
        maximum_bytes=4 * 1024 * 1024,
    ),
    Asset(
        "splash-master.jpg",
        "splash.png",
        1920,
        1080,
        maximum_bytes=8 * 1024 * 1024,
    ),
)


def crop_to_aspect(image: Image.Image, asset: Asset) -> Image.Image:
    target_aspect = asset.width / asset.height
    source_aspect = image.width / image.height
    if source_aspect > target_aspect:
        crop_width = round(image.height * target_aspect)
        crop_height = image.height
    else:
        crop_width = image.width
        crop_height = round(image.width / target_aspect)

    left = round((image.width - crop_width) * asset.center_x)
    top = round((image.height - crop_height) * asset.center_y)
    left = max(0, min(left, image.width - crop_width))
    top = max(0, min(top, image.height - crop_height))
    return image.crop((left, top, left + crop_width, top + crop_height))


def build(asset: Asset) -> Path:
    source_path = SOURCE / asset.source
    if not source_path.is_file():
        raise FileNotFoundError(f"missing RF-5 branding master: {source_path}")

    with Image.open(source_path) as source:
        mode = "RGBA" if asset.preserve_alpha else "RGB"
        image = ImageOps.exif_transpose(source).convert(mode)
        image = crop_to_aspect(image, asset)
        image = image.resize((asset.width, asset.height), Image.Resampling.LANCZOS)
        # Seven effective bits per channel substantially reduce photographic
        # PNG entropy while preserving the satin black and walnut gradients.
        if asset.preserve_alpha:
            alpha = image.getchannel("A")
            image = ImageOps.posterize(image.convert("RGB"), 7)
            image.putalpha(alpha)
        else:
            image = ImageOps.posterize(image, 7)

    OUTPUT.mkdir(parents=True, exist_ok=True)
    output_path = OUTPUT / asset.output
    image.save(output_path, format="PNG", optimize=True, compress_level=9)
    size = output_path.stat().st_size
    if size > asset.maximum_bytes:
        raise ValueError(
            f"{output_path} is {size} bytes; RackForge allows {asset.maximum_bytes}"
        )
    print(
        f"BRANDING_WRITTEN path={output_path} "
        f"size={asset.width}x{asset.height} mode={image.mode} bytes={size}"
    )
    return output_path


def main() -> None:
    for asset in ASSETS:
        build(asset)


if __name__ == "__main__":
    main()
