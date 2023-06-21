; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,114
lps $80
  mov $0,$$81
  mov $1,$0
  mov $2,$$83
  f21 $1,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
  mov $2,1
  mov $3,1
  mov $4,2
  f41 $1,102210 ; Draw lines between the `color0` pixels and `color1` pixels when both occur in the same column/row.
  mov $2,2
  mov $3,$2
  mov $4,3
  f41 $1,102210 ; Draw lines between the `color0` pixels and `color1` pixels when both occur in the same column/row.
  mov $2,3
  f21 $1,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
  mov $2,$0
  mov $3,254
  f31 $1,102131 ; Pick pixels from image and color. When the mask is 0 then pick from the image. When the mask is [1..255] then use the `default_color`.
  mov $0,$1
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe

; template: "1d_scale_dp_2-1.asm"
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: SetSourceToDirect
