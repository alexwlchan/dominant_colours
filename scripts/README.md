This folder has some scripts I've written that use the output of dominant colours to do interesting things.

## dominant_slices.py

This "slices" an image based on its dominant colours: each slice contains then pixels of the original image which are closest to one of the dominant colours.

```console
$ python3 dominant_slices.py lighthouse.jpg --max-colours=5
```

Example:

<table>
  <tr>
    <td>
      #e8e3d7
      <img src="lighthouse.e8e3d7.png">
    </td>
    <td>
      #838882
      <img src="lighthouse.838882.png">
    </td>
    <td>
      #4576bb
      <img src="lighthouse.4576bb.png">
    </td>
    <td>
      #292019
      <img src="lighthouse.292019.png">
    </td>
    <td>
      #c53b4e
      <img src="lighthouse.c53b4e.png">
    </td>
  </tr>
</table>
