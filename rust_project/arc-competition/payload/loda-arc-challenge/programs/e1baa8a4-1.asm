; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,3
  f21 $0,101092 ; Denoise type3. denoise noisy pixels. Takes a 2nd parameter: number of repair iterations.
  f11 $0,101140 ; Image: Remove duplicate rows/columns
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "5614dbcf-1.asm"
; mutate: ReplaceLineWithHistogram
; mutate: ReplaceLineWithHistogram
