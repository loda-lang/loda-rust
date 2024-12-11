; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,108 ; address of vector[0].GridMask
mov $84,109 ; address of vector[0].GridColor
lps $80
  mov $0,$$81
  mov $1,$$84 ; grid color
  f21 $0,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.
  mov $1,$$83 ; grid mask
  f21 $0,101254 ; xor
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
  add $84,100
lpe
