; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,114 ; address of vector[0].InputMostPopularColor
lps $80
  mov $20,$$83 ; most popular color across inputs

  mov $10,$$81 ; input image
  mov $11,0 ; no spacing
  f24 $10,102261 ; split into 4 rows
  ; $10..$13 are the 4 rows

  mov $0,$20 ; transparent color
  mov $1,$12 ; layer 0 lowest layer
  mov $2,$13 ; layer 1
  mov $3,$11 ; layer 2
  mov $4,$10 ; layer 3 top
  f51 $0,101152 ; Z-stack images: Overlay multiple images using a transparency color

  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
