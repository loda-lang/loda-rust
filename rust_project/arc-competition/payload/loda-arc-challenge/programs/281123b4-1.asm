; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,114 ; address of vector[0].InputMostPopularColor
lps $80
  mov $0,$$81 ; input image
  mov $20,$$83 ; most popular color across inputs

  mov $1,1 ; spacing is 1 pixel
  f24 $0,102260 ; split into 4 columns
  ; $0..$3 are the 4 columns

  mov $10,$20 ; transparent color
  mov $11,$1 ; layer 0 lowest layer
  mov $12,$0 ; layer 1
  mov $13,$3 ; layer 2
  mov $14,$2 ; layer 3 top
  f51 $10,101152 ; Z-stack images: Overlay multiple images using a transparency color

  mov $0,$10
  
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
