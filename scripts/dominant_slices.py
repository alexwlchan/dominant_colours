#!/usr/bin/env python3
"""
Given an image, this divides it into "slices" based on its dominant colour
components.

It starts by finding the dominant colours, then groups the pixels based
on which colour they're closest to.  There's one slice per colour -- containing
all the pixels which are closest to this colour.

"""

import argparse
import os
import subprocess

from PIL import Image
import tqdm


def parse_hex(line):
    return (int(line[1:3], 16), int(line[3:5], 16), int(line[5:7], 16))


def parse_args():
    parser = argparse.ArgumentParser(
        description="Slice an image based on its dominant colours."
    )
    parser.add_argument("path", metavar="PATH", help="Path to the image to slice")
    parser.add_argument(
        "--max-colours",
        dest="max_colours",
        default=5,
        help="how many colours to find",
    )

    return parser.parse_args()


def get_dominant_colours(args):
    """
    Finds the dominant colours given the argumrnt.

    Returns a list of RGB tuples, e.g.

        [
            (233, 228, 215),
            (133, 139, 136),
            (69, 118, 187),
        ]

    """
    command = [
        "dominant_colours",
        "--no-palette",
        args.path,
        f"--max-colours={args.max_colours}",
    ]
    output = subprocess.check_output(command)

    return [parse_hex(line) for line in output.decode("ascii").strip().splitlines()]


if __name__ == "__main__":
    args = parse_args()

    dominant_colours = get_dominant_colours(args)

    new_image_data = {colour: [] for colour in dominant_colours}

    im = Image.open(args.path)

    for pixel in tqdm.tqdm(im.getdata()):
        if im.mode == "RGB":
            r, g, b = pixel
            alpha = 255
        elif im.mode == "RGBA":
            r, g, b, alpha = pixel
        elif im.mode == "L":
            r = g = b = pixel
            alpha = 255
        else:
            raise ValueError(f"Unsupported image mode: {im.mode}")

        closest_colour = min(
            dominant_colours,
            key=lambda c: (r - c[0]) ** 2 + (g - c[1]) ** 2 + (b - c[2]) ** 2,
        )

        for colour in new_image_data:
            if colour == closest_colour:
                new_image_data[colour].append((r, g, b, alpha))
            else:
                new_image_data[colour].append((255, 255, 255, 0))

    for (r, g, b), data in new_image_data.items():
        new_im = Image.new("RGBA", im.size)
        new_im.putdata(data)

        name, _ = os.path.splitext(os.path.basename(args.path))
        out_path = f"{name}.{r:02x}{g:02x}{b:02x}.png"

        new_im.save(out_path)
