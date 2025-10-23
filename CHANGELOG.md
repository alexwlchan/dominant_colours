# Changelog

## v1.5.0 - 2025-10-23

*   Add support for AVIF images.

## v1.4.1 - 2024-11-06

*   Fix a bug introduced in v1.3.0 where getting colours for non-animated WebP images would fail with an assertion error.

## v1.4.0 - 2024-10-05

*   `dominant_colours` will now skip printing terminal colours if it detects it's not running in a tty.  This makes it slightly easier to use in automated environments, because you don't need to pass the `--no-palette` flag.

## v1.3.0 - 2024-09-04

*   Add support for animated WebP images.
*   Improve the error messages, especially when dealing with malformed images.

## v1.2.0 - 2024-05-12

Two new features:

*   Add support for WebP images.
*   Add an experimental new flag `--best-against-bg=[HEX_COLOUR]` that will print the single colour which will look best against this background.

## v1.1.9 - 2024-05-12

Bump the version of all the dependency libraries, and try to get the release process working again.

This has no feature changes.

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
