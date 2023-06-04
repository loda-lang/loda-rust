; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
lps $80
  mov $0,$$81 ; input image
  mov $1,0
  mov $2,0
  mov $3,0
  mov $4,4 ; grow right 4 times
  f51 $0,102122 ; repeat symmetry
  mov $$82,$0
  add $81,100
  add $82,100
lpe
