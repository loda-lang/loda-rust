; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,110
mov $82,102
lps $80
  mov $0,$$81
  mov $1,5
  mov $2,0
  f31 $0,104201 ; Group the objects into 2 bins based on mass: objects that has the matching mass=1, objects that have a different mass=2.
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "d2abd087-1.asm"
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: DecrementSourceValueWhereTypeIsConstant
