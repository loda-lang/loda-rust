; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,115 ; address of vector[0].InputSinglePixelNoiseColor
lps $80
  mov $0,$$81 ; input image

  mov $1,$0
  mov $2,$$83 ; noise color
  f21 $1,101250 ; mask where color is noise color

  ; $1 = image
  mov $2,1 ; color0 = 1
  mov $3,1 ; color1 = 1
  mov $4,2 ; line_color = 2
  f41 $1,102210 ; Draw lines between the `color0` pixels and `color1` pixels when both occur in the same column/row.

  ; $1 = image
  mov $2,2 ; color0 = 2
  mov $3,2 ; color1 = 2
  mov $4,3 ; line_color = 3
  f41 $1,102210 ; Draw lines between the `color0` pixels and `color1` pixels when both occur in the same column/row.

  mov $2,3
  f21 $1,101250 ; mask where color is 3

  mov $2,$0
  mov $3,255
  f31 $1,102131 ; Pick pixels from image and color. When the mask is 0 then pick from the image. When the mask is [1..255] then use the `default_color`.

  mov $0,$1

  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
