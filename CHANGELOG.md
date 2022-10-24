# Changelog

## v1.2.0 - 2022-10-24

Add a new flag `--compared-to` which allows you to specify a background colour you're going to use.
`dominant_colours` will then select the best colour to use, based on two criteria:

*   Whether the colour has enough contrast with the background (looking for a pass with WCAG AA)
*   Maximising the saturation, to select for bright and fun colours

Example:

```console
$ cargo run -- lighthouse.jpg --compared-to='#000000'
▇ #4880cd  # blue

$ cargo run -- lighthouse.jpg --compared-to='#ffffff'
▇ #c53b4e  # red
```

## v1.1.8 - 2022-10-24

More internal refactoring to use newer versions of kmeans-colors and palette.

This has no feature changes.

## v1.1.7 - 2022-10-24

Some internal refactoring to use a newer versions of clap.

This has no feature changes.

## v1.1.6 - 2022-10-23

Provide precompiled binaries for more targets, so the following targets are now supported:

* Macs on Intel and Apple Silicon
* Windows on Intel
* Linux on Intel, both GNU and MUSL (Alpine-compatible) binaries

There are no feature/bugfix changes from v1.1.3.

(Note: v1.1.4 and v1.1.5 were abortive releases to get the new binaries working, and have been removed to avoid confusion.

## v1.1.3 - 2022-10-17

Fix a bug when finding the dominant colour of some animated GIFs.

## v1.1.2 - 2022-10-16

Add support for TIFF images.

## v1.1.1 - 2022-04-03

Publish binaries as part of the GitHub releases.

This has no feature changes.

## v1.1.0 - 2021-11-29

Allow finding the dominant colour of GIFs.

This includes animated GIFs, and dominant_colours looks at multiple frames.

## v1.0.1 - 2021-11-28

Use uppercase text for variables in the help text.
i.e.

```
dominant_colours <PATH> --max-colours <MAX-COLOURS>
```

rather than

```
dominant_colours <path> --max-colours <max-colours>
```

## v1.0.0 - 2021-11-27

Initial release.
