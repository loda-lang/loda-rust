; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,114 ; address of vector[0].InputMostPopularColor
lps $80
  mov $0,$$81 ; input image
  mov $20,$$83 ; most popular color across inputs

  mov $1,$0
  mov $2,$20
  f21 $1,101251 ; where color is different than the most popular color

  mov $1,$1 ; mask
  mov $2,255 ; overlap color
  f31 $0,102223 ; Shoot out lines in all directions where mask is non-zero. Preserving the color.

  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
