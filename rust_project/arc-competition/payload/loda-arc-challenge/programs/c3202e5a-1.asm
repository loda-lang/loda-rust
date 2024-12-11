; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,110 ; address of vector[0].EnumeratedObjects
lps $80
  mov $0,$$81

  mov $1,$$83 ; enumerated objects
  f21 $0,104000 ; Count unique colors in each object

  ; $1 is the enumerated objects
  f21 $0,104100 ; Pick object with the smallest value

  mov $1,255 ; color for the area to be trimmed
  mov $2,$$81
  f31 $0,102130 ; Pick pixels from color and image

  ; $1 is the color to be trimmed
  f21 $0,101161 ; trim with color

  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
