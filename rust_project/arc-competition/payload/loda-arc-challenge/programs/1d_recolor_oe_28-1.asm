; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,110
mov $82,102
mov $83,114
lps $80
  mov $$82,$$81
  add $81,100
  add $82,100
lpe

; template: "enumerated-objects.asm"
; mutate: ReplaceLineWithHistogram
; mutate: SwapRegisters, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram
