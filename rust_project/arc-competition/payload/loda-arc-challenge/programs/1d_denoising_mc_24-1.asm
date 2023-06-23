; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,114
lps $80
  mov $0,$$81
  mov $1,$$83
  mov $3,255
  mov $2,5
  f41 $0,102210 ; Draw lines between the `color0` pixels and `color1` pixels when both occur in the same column/row.
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe

; template: "AboveBelow8-1.asm"
; mutate: InsertLineWithHistogram, no change
; mutate: SwapAdjacentRows
; mutate: IncrementSourceValueWhereTypeIsDirect
; mutate: CallLeastPopularProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: SetSourceToConstant
