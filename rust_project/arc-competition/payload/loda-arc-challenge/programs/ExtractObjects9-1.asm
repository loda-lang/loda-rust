; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $2,0
  mov $1,$0
  f11 $1,101070 ; Image the 1 most unpopular colors, sorted by unpopularity
  mov $2,0
  f31 $0,101051 ; Image: replace colors other than x with color y
  mov $1,102
  f11 $0,101160 ; Image: Trim border using histogram of border pixels
  f31 $0,101080 ; Image: Draw outline around things that aren't the background
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "31aa019c-1.asm"
; mutate: InsertLineWithHistogram
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: CallMostPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceTargetWithHistogram, no change
; mutate: SwapRows
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: ReplaceSourceConstantWithHistogram
