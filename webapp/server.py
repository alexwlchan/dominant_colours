#!/usr/bin/env python

import base64
import os
import secrets
import subprocess
import sys
import tempfile

from flask import Flask, flash, redirect, render_template, request
import wcag_contrast_ratio as contrast


app = Flask(__name__)
app.secret_key = secrets.token_hex()


VERSION = subprocess.check_output(["dominant_colours", "--version"]).decode("utf8")


@app.route("/")
def index():
    return render_template("index.html", version=VERSION)


@app.template_filter("usable_colours")
def usable_colours(colours):
    result = []

    for hex_string in colours:
        r = red(hex_string)
        g = green(hex_string)
        b = blue(hex_string)

        ratio = contrast.rgb((r / 255, g / 255, b / 255), (1, 1, 1))

        if contrast.passes_AA(ratio):
            result.append(hex_string)

    return result


@app.template_filter("red")
def red(hex_string):
    return int(hex_string[1:3], 16)


@app.template_filter("green")
def green(hex_string):
    return int(hex_string[3:5], 16)


@app.template_filter("blue")
def blue(hex_string):
    return int(hex_string[5:7], 16)


@app.route("/palette", methods=["GET", "POST"])
def get_palette():
    if request.method == "GET":
        return redirect("/")

    if request.method == "POST":
        uploaded_file = request.files["file"]
        _, extension = os.path.splitext(uploaded_file.filename)

        with tempfile.NamedTemporaryFile(suffix=extension) as tmp_file:
            uploaded_file.save(tmp_file)

            # If we don't flush here, the file may be incomplete.  This can
            # lead to errors like:
            #
            #     failed to fill whole buffer
            #
            # when running dominant_colours.
            tmp_file.flush()

            proc = subprocess.Popen(
                ["dominant_colours", tmp_file.name, "--no-palette", "--max-colours=5"],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            stdout, stderr = proc.communicate()
            return_code = proc.poll()

            if return_code != 0:
                stderr = stderr.decode(sys.stdin.encoding)
                flash(f"Something went wrong:<br/>{stderr}")
                return redirect("/")

            stdout = stdout.decode(sys.stdin.encoding)
            colours = stdout.strip().split("\n")

            with tempfile.NamedTemporaryFile(suffix="jpg") as thumbnail_file:
                subprocess.check_call(
                    [
                        "convert",
                        tmp_file.name,
                        "-resize",
                        "600x600",
                        thumbnail_file.name,
                    ]
                )
                thumbnail_file.seek(0)
                thumbnail = thumbnail_file.read()

            thumbnail_data_uri = (
                b"data:image/jpg;base64," + base64.b64encode(thumbnail)
            ).decode("ascii")

            return render_template(
                "palette.html",
                colours=colours,
                thumbnail_data_uri=thumbnail_data_uri,
                version=VERSION,
            )


if __name__ == "__main__":
    app.run(debug=True, host="0.0.0.0", port=4711)
