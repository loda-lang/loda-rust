; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,105 ; address of vector[0].OutputImageIsInputImageWithChangesLimitedToPixelsWithColor
lps $80
  mov $0,$$81

  mov $4,$0
  f11 $4,102140 ; Traverse all pixels in the 3x3 convolution and count how many have the same color as the center.

  mov $5,$0
  mov $6,$$83
  f21 $5,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

  mov $8,42
  mov $7,$4
  mov $6,$5
  f31 $6,102131 ; Pick pixels from color and image. When the mask is 0 then pick the `default_color`. When the mask is [1..255] then pick from the image.

  mov $$82,$6
  add $81,100
  add $82,100
  add $83,100
lpe
