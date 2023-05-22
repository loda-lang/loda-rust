; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,115 ; address of vector[0].InputSinglePixelNoiseColor
lps $80
  mov $0,$$81 ; input image

  mov $8,$0
  mov $9,$$83 ; noise color
  f21 $8,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
  ; $8 is now the mask

  mov $10,$0 ; image
  mov $11,$8 ; mask
  mov $12,42 ; line color
  f31 $10,102222 ; Draw a vertical line if the `mask` contains one or more non-zero pixels.
  ; $10 is now the columns image

  mov $13,$8 ; mask
  mov $14,$10 ; the columns image
  mov $15,$0 ; input image
  f31 $13,102132 ; Pick pixels from two images. When the mask is 0 then pick `image_a`. When the mask is [1..255] then pick from `image_b`.

  mov $0,$13
  mov $1,2
  mov $2,2
  f31 $0,102120 ; repeat image

  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
