; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,114 ; address of vector[0].InputMostPopularColor
mov $84,115 ; address of vector[0].InputSinglePixelNoiseColor
lps $80
  mov $0,$$81 ; input image
  mov $1,$$84 ; noise color
  mov $2,$$83 ; background color
  f31 $0,101093 ; Denoise type4. denoise noisy pixels.
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
  add $84,100
lpe
