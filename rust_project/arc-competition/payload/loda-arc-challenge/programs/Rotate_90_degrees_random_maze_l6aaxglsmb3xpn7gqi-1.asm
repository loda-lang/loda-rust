; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,1
  f21 $0,101170 ; Image: Rotate by x * 90 degrees
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "6150a2bd-1.asm"
; mutate: CallMediumPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: DecrementSourceValueWhereTypeIsConstant
