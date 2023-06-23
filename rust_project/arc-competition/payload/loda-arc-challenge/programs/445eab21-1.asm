; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,$0
  f11 $1,101060 ; Image the 1 most popular colors, sorted by popularity
  f21 $0,101110 ; Image: detect holes. Takes a color parameter for the empty areas.
  mov $2,$0
  f11 $2,101070 ; Image the 1 most unpopular colors, sorted by unpopularity
  mov $0,2
  mov $1,2
  f31 $0,101010 ; Create new image with size (x, y) and filled with color z
  mov $$82,$0
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "b9b7f026-1.asm"
; mutate: IncrementSourceValueWhereTypeIsConstant
; mutate: IncrementTargetValueWhereTypeIsDirect
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CopyLine
; mutate: IncrementSourceValueWhereTypeIsConstant
