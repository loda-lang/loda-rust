; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,105
lps $80
  mov $0,$$81
  mov $1,$$83
  f21 $0,102151 ; Repair damaged pixels and recreate big repeating patterns such as mosaics.
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
  add $84,100
lpe

; template: "repair-mosaic.asm"
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram
