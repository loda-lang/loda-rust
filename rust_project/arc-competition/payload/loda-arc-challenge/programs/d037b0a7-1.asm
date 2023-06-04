; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,114 ; address of vector[0].InputMostPopularColor
lps $80
  mov $0,$$81 ; input image
  mov $20,$$83 ; most popular color across inputs

  ; Construct ignore_mask based on the most popular color
  mov $1,$0
  mov $2,$20
  f21 $1,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
  ; $1 is the pixels to be ignored

  ; determine the nearest color in the direction 'up'
  mov $3,$0
  mov $4,$1 ; ignore mask
  mov $5,$20 ; color_when_there_is_no_neighbour
  f31 $3,102060 ; color of nearest neighbour pixel 'up'
  ; $3 is an image of the nearest color in the direction 'up'

  ; combine images based on the ignore mask
  mov $6,$1
  mov $7,$0
  mov $8,$3
  f31 $6,102132 ; Pick pixels from two images. When the mask is 0 then pick `image_a`. When the mask is [1..255] then pick from `image_b`.
  ; $6 is a combination of the original image and the nearest color in the direction 'up'
  mov $0,$6

  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
