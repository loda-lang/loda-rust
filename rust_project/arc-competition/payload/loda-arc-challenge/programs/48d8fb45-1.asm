; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  f11 $0,102000 ; Image: Extracts the most popular object.
  mov $$82,$0
  add $81,10
  add $82,10
lpe

; template: "3c9b0459-1.asm"
; mutate: CallLeastPopularProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallMediumPopularProgram, no change
; mutate: DecrementSourceValueWhereTypeIsDirect, no change
; mutate: IncrementTargetValueWhereTypeIsDirect
; mutate: InsertLineWithHistogram, no change
; mutate: ReplaceLineWithHistogram
