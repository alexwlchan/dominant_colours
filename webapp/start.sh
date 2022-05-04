#!/usr/bin/env bash

set -o errexit

pip3 install -r requirements.txt

DOWNLOAD_URL="https://github.com/alexwlchan/dominant_colours/releases/download/v1.1.1/dominant_colours-x86_64-unknown-linux-musl.tar.gz"

# Remove any existing downloads
rm -f dominant_colours*

# The --location flag means we follow redirects
curl --location "$DOWNLOAD_URL" > dominant_colours.tar.gz
md5sum dominant_colours.tar.gz
tar -xzf dominant_colours.tar.gz

chmod +x dominant_colours
md5sum dominant_colours
./dominant_colours --version

# Make sure we can find dominant_colours in the app
export PATH=$(pwd):$PATH

if [[ "$DEBUG" == "yes" ]]
then
  python3 server.py
else
  gunicorn server:app -w 4 --log-file -
fi
