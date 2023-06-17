; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $82,102
mov $83,106
mov $84,100
lps $80
  mov $0,$$83
  mov $1,255
  mov $2,$$84
  f31 $0,102130 ; Pick pixels from color and image. When the mask is 0 then pick the `default_color`. When the mask is [1..255] then pick from the image.
  mov $1,255
  f21 $0,101161 ; Image: Trim border with color to be trimmed
  mov $$82,$0
  add $82,100
  add $83,100
  add $84,100
lpe

; template: "repair-symmetry-and-crop.asm"
; mutate: ReplaceLineWithHistogram, no change
; mutate: ReplaceSourceConstantWithHistogram
