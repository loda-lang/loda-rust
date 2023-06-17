; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,100
  mov $2,1
  f31 $0,101180 ; Adjust image offset(dx, dy) with wrap
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "a79310a0-1.asm"
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: SetSourceToConstant, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceInstructionWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: ReplaceSourceConstantWithHistogram
