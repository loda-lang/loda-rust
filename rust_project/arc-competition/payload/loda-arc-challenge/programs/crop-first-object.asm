; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,110 ; address of vector[0].EnumeratedObjects
lps $80
  ; extract the background image
  mov $1,$$83 ; enumerated objects
  mov $2,0 ; the 0th object is the background object
  f21 $1,101250 ; where color is the mask of the 0th object

  ; histogram of the background image
  mov $0,$$81 ; input image
  f21 $0,101231 ; histogram with mask

  ; get pixel at x=0, y=1, this is the most popular color
  mov $1,0
  mov $2,1
  f31 $0,101002  ; get pixel of the most popular color
  mov $10,$0

  ; extract object 1, the biggest object
  mov $0,$$83 ; enumerated objects
  mov $1,1 ; the 1st object is the biggest object
  f21 $0,101250 ; where color is the mask of the 1st object

  ; surround the object with the background-color
  mov $1,$10 ; color for the area to be trimmed
  mov $2,$$81
  f31 $0,102130 ; Pick pixels from color and image

  ; $1 is the color to be trimmed
  f21 $0,101161 ; trim with color

  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
