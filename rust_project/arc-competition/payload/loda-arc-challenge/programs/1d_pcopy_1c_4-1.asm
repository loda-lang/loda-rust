; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,111
add $82,100
mov $82,102
lps $80
  mov $$82,$$81
  add $81,100
  add $82,100
lpe

; template: "enumerated-objects.asm"
; mutate: InsertLineWithHistogram, no change
; mutate: IncrementSourceValueWhereTypeIsConstant
; mutate: CopyLine
