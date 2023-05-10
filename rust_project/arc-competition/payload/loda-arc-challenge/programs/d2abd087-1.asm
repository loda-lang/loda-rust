; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,110 ; address of vector[0].EnumeratedObjects
mov $82,102 ; address of vector[0].ComputedOutputImage
lps $80
  mov $0,$$81 ; enumerated objects
  mov $1,6 ; objects with 'mass = 6'
  mov $2,0 ; reverse = false
  f31 $0,104201 ; Group the objects into 2 bins based on mass: objects that has the matching mass=1, objects that have a different mass=2.
  mov $$82,$0
  add $81,100
  add $82,100
lpe
