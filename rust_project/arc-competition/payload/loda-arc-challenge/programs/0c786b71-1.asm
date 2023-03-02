; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,$0
  f11 $0,101190 ; Image: flip x
  f21 $0,101030 ; Image.hstack. horizontal stack of 2 images
  mov $1,$0
  f11 $0,101191 ; Image: flip y
  f21 $0,101040 ; Image.vstack. vertical stack of 2 images
  mov $$82,$0
  add $81,10
  add $82,10
lpe

; template: "3af2c5a8-1.asm"
; mutate: InsertLineWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceInstructionWithHistogram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallMediumPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: DecrementTargetValueWhereTypeIsDirect
