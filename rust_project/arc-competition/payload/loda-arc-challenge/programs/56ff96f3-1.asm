; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,114 ; address of vector[0].InputMostPopularColor
lps $80
  mov $0,$$81 ; input image
  mov $1,$$83 ; background color
  f21 $0,102250 ; Draw non-overlapping filled rectangles over the bounding boxes of each color
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
