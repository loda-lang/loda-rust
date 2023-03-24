; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  f11 $0,101160 ; Image: Trim border using histogram of border pixels
  f11 $0,101140 ; Image: Remove duplicate rows/columns
  f11 $0,101120 ; Image: remove grid patterns.
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "90c28cc7-1.asm"
; mutate: InsertLineWithHistogram, no change
; mutate: SwapRegisters, no change
; mutate: CallLeastPopularProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: InsertLineWithHistogram
