; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,110
lps $80
  mov $1,$$83
  mov $2,0
  f21 $1,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
  mov $0,$$81
  f11 $0,101230 ; Histogram of image. The most popular to the left, least popular to the right. The top row is the counters. The bottom row is the colors.
  mov $1,0
  mov $2,1
  f31 $0,101002 ; Image: get pixel at (x, y)
  mov $10,$0
  mov $0,$$83
  mov $1,1
  f21 $0,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
  mov $1,$10
  mov $2,$$81
  f31 $0,102130 ; Pick pixels from color and image. When the mask is 0 then pick the `default_color`. When the mask is [1..255] then pick from the image.
  f21 $0,101161 ; Image: Trim border with color to be trimmed
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe

; template: "crop-first-object.asm"
; mutate: ReplaceLineWithHistogram
; mutate: ReplaceLineWithHistogram
