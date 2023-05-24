; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,105 ; address of vector[0].OutputImageIsInputImageWithChangesLimitedToPixelsWithColor
lps $80
  mov $0,$$81 ; input image
  mov $1,$$83 ; from_color = changes limited to this color
  mov $2,255 ; to_color = auto assign color
  f31 $0,102180 ; Flood fill at every pixel along the border, connectivity-4.
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
