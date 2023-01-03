; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  f11 $0,101120 ; Image: remove grid patterns.
  f11 $1,101140 ; Image: Remove duplicate rows/columns
  mov $$82,$0
  add $81,10
  add $82,10
lpe

; template: "90c28cc7-1.asm"
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: SetSourceToDirect, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceSourceWithHistogram, no change
; mutate: IncrementSourceValueWhereTypeIsDirect, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: IncrementTargetValueWhereTypeIsDirect
