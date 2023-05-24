; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,115 ; address of vector[0].InputSinglePixelNoiseColor
lps $80
  mov $0,$$81 ; input image
  mov $1,$$83 ; color0 = noise color
  mov $2,$1 ; color1 = noise color
  mov $3,$1 ; line_color = noise color
  f41 $0,102210 ; Draw lines between the `color0` pixels and `color1` pixels when both occur in the same column/row.
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
