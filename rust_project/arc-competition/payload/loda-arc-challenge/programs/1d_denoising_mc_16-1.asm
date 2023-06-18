; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,3
  f21 $0,101092 ; Denoise type3. denoise noisy pixels. Takes a 2nd parameter: number of repair iterations.
  mov $1,100
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "eb5a1d5d-2.asm"
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
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: IncrementSourceValueWhereTypeIsDirect, no change
; mutate: CallRecentProgram, no change
; mutate: SwapRows
; mutate: ReplaceLineWithHistogram
