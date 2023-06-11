; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,114
lps $80
  mov $0,$$81
  mov $2,100
  f21 $0,102193 ; Gravity in the right direction
  add $83,100
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "beb8660c-1.asm"
; mutate: CallRecentProgram, no change
; mutate: SetSourceToConstant
; mutate: ReplaceLineWithHistogram
; mutate: ReplaceLineWithHistogram, no change
; mutate: CallMostPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallMediumPopularProgram, no change
; mutate: CallMediumPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: DecrementSourceValueWhereTypeIsDirect, no change
; mutate: SwapAdjacentRows
; mutate: IncrementTargetValueWhereTypeIsDirect
