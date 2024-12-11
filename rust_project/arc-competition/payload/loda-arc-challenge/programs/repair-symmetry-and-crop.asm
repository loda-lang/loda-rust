; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,106 ; address of vector[0].RepairMask
mov $84,107 ; address of vector[0].RepairedImage
lps $80
  ; replace what is outside the repair mask with the color 255
  mov $0,$$83 ; repair mask
  mov $1,255 ; color for what to be removed
  mov $2,$$84 ; repaired image
  f31 $0,102130 ; Pick pixels from color and image. When the mask is 0 then pick the `default_color`. When the mask is [1..255] then pick from the image.

  ; crop out the repair mask
  mov $1,255
  f21 $0,101161 ; trim with color

  mov $$82,$0
  add $82,100
  add $83,100
  add $84,100
lpe
