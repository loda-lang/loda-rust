; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
lps $80
  mov $0,$$81 ; input image

  mov $1,$0
  f11 $1,101240 ; Number of unique colors in image.

  mov $2,$1
  f31 $0,102120 ; Repeat image by the number of unique colors in image.

  mov $$82,$0
  add $81,100
  add $82,100
lpe
