; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,114 ; address of vector[0].InputMostPopularColor
lps $80
  mov $0,$$81 ; input image
  mov $20,$$83 ; most popular color across inputs

  ; determine if the image needs to be flipped
  mov $8,$0
  mov $9,1
  f21 $8,101222 ; get N left columns
  mov $9,$20 ; most popular color
  f21 $8,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
  f11 $8,101243 ; number of zeroes in image
  mod $8,2
  ; $8 is 1 when the input image has its content on the right side, and needs flipping. Otherwise it's 0.

  mov $9,$8

  ; flip the input image so it's content is on the right side
  lps $8
    f11 $0,101190 ; flip x
  lpe

  mov $1,$0
  f11 $1,101190 ; flip x
  f21 $1,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
  f21 $0,101030 ; hstack

  ; restore the x axis
  lps $9
    f11 $0,101190 ; flip x
  lpe
  
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
