; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,114 ; address of vector[0].InputMostPopularColor
lps $80
  mov $0,$$81 ; input image
  mov $1,$$83 ; most popular color across inputs
  f21 $0,102193 ; Gravity in the right direction

  ; $1 holds the most popular color across inputs
  f21 $0,102200 ; Sort rows-ascending by color

  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
