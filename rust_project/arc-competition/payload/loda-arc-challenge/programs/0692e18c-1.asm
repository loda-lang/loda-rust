; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,114 ; address of vector[0].InputMostPopularColor
lps $80
  mov $0,$$81 ; input image
  mov $20,$$83 ; most popular color across inputs

  ; tile_width
  mov $2,$0
  f11 $2,101000 ; Get width of image

  ; tile_height
  mov $3,$0
  f11 $3,101001 ; Get height of image

  ; tile0
  mov $7,$20 ; color
  mov $6,$3 ; height
  mov $5,$2 ; width
  f31 $5,101010 ; Create new image with size (x, y) and filled with color z

  ; mask
  mov $10,$0 ; image
  mov $11,$20 ; color
  f21 $10,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

  ; tile1
  f11 $0,102170 ; Reorder the color palette, so that the `most popular color` changes place with the `least popular color`

  mov $11,$5 ; tile0
  mov $12,$0 ; tile1
  f31 $10,102110 ; Create a big composition of tiles.

  mov $0,$10

  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
