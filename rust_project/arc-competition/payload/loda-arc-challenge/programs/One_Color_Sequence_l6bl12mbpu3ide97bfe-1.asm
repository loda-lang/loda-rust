; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,114
lps $80
  mov $0,$$81
  mov $1,$$83
  f21 $0,102193 ; Gravity in the right direction
  f21 $0,102200 ; Sort rows-ascending by mass
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
  mov $81,100
lpe

; template: "beb8660c-1.asm"
; mutate: SwapRegisters, no change
; mutate: CopyLine
