; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,98
  mov $2,1
  f31 $0,101180 ; Adjust image offset(dx, dy) with wrap
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "1d_move_3p_15-1.asm"
; mutate: CallMediumPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: DecrementSourceValueWhereTypeIsConstant
