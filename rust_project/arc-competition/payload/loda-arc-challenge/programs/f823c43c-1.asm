; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,6
  f21 $0,102151 ; Repair damaged pixels and recreate big repeating patterns such as mosaics.
  mov $$82,$0
  add $81,10
  add $82,10
lpe

; template: "ea959feb-1.asm"
; mutate: ReplaceSourceWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: SwapRegisters, no change
; mutate: CallMediumPopularProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: IncrementTargetValueWhereTypeIsDirect
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: DecrementSourceValueWhereTypeIsConstant
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: IncrementSourceValueWhereTypeIsDirect, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceInstructionWithHistogram, no change
; mutate: ReplaceLineWithHistogram
; mutate: InsertLineWithHistogram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: SwapRegisters, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceSourceConstantWithHistogram
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallMediumPopularProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallMediumPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: SwapAdjacentRows
; mutate: SwapAdjacentRows
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: IncrementSourceValueWhereTypeIsConstant
