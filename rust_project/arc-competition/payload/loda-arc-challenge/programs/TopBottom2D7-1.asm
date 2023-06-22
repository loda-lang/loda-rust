; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,110 ; address of vector[0].EnumeratedObjects
lps $80
  mov $0,$$83 ; enumerated objects
  f11 $0,104210 ; mask of top-most object
  
  ; $0 = mask of top-most object
  mov $1,255 ; fill color
  mov $2,$$81 ; input image
  f31 $0,102130 ; Pick pixels from color and image. When the mask is 0 then pick the `default_color`. When the mask is [1..255] then pick from the image.

  mov $1,255 ; the color to be trimmed
  f21 $0,101161 ; trim with color

  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
