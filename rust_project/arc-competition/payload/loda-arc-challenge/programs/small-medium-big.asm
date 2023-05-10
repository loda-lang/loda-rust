; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,110 ; address of vector[0].EnumeratedObjects
mov $82,102 ; address of vector[0].ComputedOutputImage
lps $80
  mov $0,$$81 ; enumerated objects
  mov $1,0 ; reverse = false
  f21 $0,104200 ; Group the objects into 3 bins based on mass: small=1, medium=2, big=3.
  mov $$82,$0
  add $81,100
  add $82,100
lpe
