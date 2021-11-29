# test images

The animated GIF was created with the following command:

```console
$ convert -delay 200 -loop 10 -dispose previous red.png blue.png red.png blue.png red.png blue.png red.png blue.png animated_squares.gif
```

The 2-second delay is to avoid causing seizures from the flashing.
I don't expect anyone else to be looking at the gif, but the delay is arbitrary so it's easy enough to avoid.
