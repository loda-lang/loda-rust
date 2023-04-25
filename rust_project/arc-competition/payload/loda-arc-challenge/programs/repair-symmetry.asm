; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,107 ; address of vector[0].RepairedImage
lps $80
  mov $$82,$$83
  add $82,100
  add $83,100
lpe
