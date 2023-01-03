; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  f11 $0,102001 ; Image: Extracts the least popular object.
  mov $$82,$0
  add $81,10
  add $82,10
lpe

; template: "1cf80156-1.asm"
; mutate: ReplaceTargetWithHistogram, no change
; mutate: ReplaceLineWithHistogram
