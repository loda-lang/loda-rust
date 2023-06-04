; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
lps $80
  mov $0,$$81 ; input image
  mov $1,$0

  f11 $0,101190 ; flip x
  f21 $0,101030 ; hstack

  mov $1,$0
  f11 $0,101191 ; flip y
  mov $2,$0
  f31 $0,101040 ; vstack

  mov $$82,$0
  add $81,100
  add $82,100
lpe
