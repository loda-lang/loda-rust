; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,112 ; address of vector[0].PredictedSingleColorImage
mov $82,102 ; address of vector[0].ComputedOutputImage
lps $80
  mov $$82,$$81
  add $81,100
  add $82,100
lpe
