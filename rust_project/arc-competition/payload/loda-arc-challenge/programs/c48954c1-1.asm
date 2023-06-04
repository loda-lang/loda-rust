; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
lps $80
  mov $0,$$81 ; input image

  mov $1,1
  mov $2,1
  mov $3,1
  mov $4,1
  f51 $0,102122 ; Make a big image by repeating the current image and doing flip x, flip y, flip xy.
  
  mov $$82,$0
  add $81,100
  add $82,100
lpe
