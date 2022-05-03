#!/usr/bin/env python

import base64
import colorsys
import os
import subprocess
import tempfile

from flask import Flask, render_template, request
import wcag_contrast_ratio as contrast


app = Flask(__name__)


@app.route("/")
def index():
    return render_template("index.html")


@app.template_filter("foreground_colour")
def foreground_colour(hex_string):
    red = int(hex_string[1:3], 16)
    green = int(hex_string[3:5], 16)
    blue = int(hex_string[5:7], 16)

    ratio = contrast.rgb((red / 255, green / 255, blue / 255), (0, 0, 0))

    if contrast.passes_AA(ratio):
        return '#000000'
    else:
        return '#FFFFFF'


@app.route("/palette", methods=["POST"])
def get_palette():
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

            result = subprocess.check_output(['dominant_colours', tmp_file.name, '--no-palette', '--max-colours=5'])
            colours = result.decode('utf8').strip().split('\n')

            with tempfile.NamedTemporaryFile(suffix='jpg') as thumbnail_file:
                subprocess.check_call(['convert', tmp_file.name, '-resize', '600x600', thumbnail_file.name])
                thumbnail_file.seek(0)
                thumbnail = thumbnail_file.read()

            thumbnail_data_uri = (b'data:image/jpg;base64,' + base64.b64encode(thumbnail)).decode('ascii')

            return render_template('palette.html', colours=colours, thumbnail_data_uri=thumbnail_data_uri)


if __name__ == "__main__":
    app.run(debug=True, host='0.0.0.0')
