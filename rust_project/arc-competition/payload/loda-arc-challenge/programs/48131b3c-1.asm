; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
lps $80
  mov $0,$$81 ; input image
  f11 $0,102170 ; Reorder the color palette, so that the `most popular color` changes place with the `least popular color`

  mov $1,2
  mov $2,2
  f31 $0,102120 ; Repeat image

  mov $$82,$0
  add $81,100
  add $82,100
lpe
