; Submitted by Simon Strandgaard
; Program Type: simple

mov $2,$0
f11 $0,102270 ; Mask, where the cells are the value is 1 and where the grid lines are the value is 0. Don't care about the color of the grid lines.
mov $1,42
f31 $0,102130 ; Pick pixels from color and image. When the mask is 0 then pick the `default_color`. When the mask is [1..255] then pick from the image.
