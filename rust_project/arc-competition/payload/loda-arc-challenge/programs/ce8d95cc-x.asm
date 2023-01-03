; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  f11 $0,101160 ; Image: Trim border using histogram of border pixels
  f11 $0,101140 ; Image: Remove duplicate rows/columns
  mov $$82,$0
  add $81,10
  add $82,10
lpe

; template: "3c9b0459-1.asm"
; mutate: DecrementSourceValueWhereTypeIsConstant, no change
; mutate: InsertLineWithHistogram, no change
; mutate: SwapRows, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: SetSourceToConstant, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: SwapAdjacentRows, no change
; mutate: InsertLineWithHistogram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallMediumPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: IncrementSourceValueWhereTypeIsConstant, no change
; mutate: CallMostPopularProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: DecrementSourceValueWhereTypeIsDirect, no change
; mutate: IncrementSourceValueWhereTypeIsConstant, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: DecrementSourceValueWhereTypeIsDirect, no change
; mutate: InsertLineWithHistogram
