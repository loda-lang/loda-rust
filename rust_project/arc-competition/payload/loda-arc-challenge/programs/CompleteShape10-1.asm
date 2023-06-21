; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,$0
  f11 $1,101190 ; Image: flip x
  f11 $1,101191 ; Image: flip y
  add $83,100
  f11 $1,101191 ; Image: flip y
  f31 $0,101150 ; Image: Overlay another image by using a color as mask
  mov $$82,$0
  add $81,100
  add $82,100
  mov $83,$0
lpe

; template: "Copy10-1.asm"
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: SwapAdjacentRows
; mutate: ReplaceTargetWithHistogram
; mutate: InsertLineWithHistogram
