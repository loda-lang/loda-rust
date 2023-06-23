; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $82,102
add $83,101
lps $80
  mov $$82,$$83
  add $82,100
  mov $84,107
lpe

; template: "repair-symmetry.asm"
; mutate: IncrementTargetValueWhereTypeIsDirect
; mutate: CallRecentProgram, no change
; mutate: ReplaceSourceWithHistogram
; mutate: SwapRows
