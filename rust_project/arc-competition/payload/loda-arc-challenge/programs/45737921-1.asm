; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,110 ; address of vector[0].EnumeratedObjects
lps $80
  mov $0,$$81 ; input image
  mov $1,$$83 ; enumerated objects
  f21 $0,102171 ; Takes 2 parameters: Image, EnumeratedObjects. Reorder the color palette, so that the `most popular color` changes place with the `least popular color`
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
