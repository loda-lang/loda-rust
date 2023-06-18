; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,114
lps $80
  mov $0,$$81
  mov $1,$0
  f11 $1,101191 ; Image: flip y
  f31 $0,101150 ; Image: Overlay another image by using a color as mask
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "CompleteShape9-1.asm"
; mutate: ReplaceLineWithHistogram
; mutate: SetSourceToDirect, no change
; mutate: IncrementSourceValueWhereTypeIsDirect
; mutate: InsertLineWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceSourceWithHistogram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallMediumPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceTargetWithHistogram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallMediumPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram
