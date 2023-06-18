; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,110
mov $82,102
lps $80
  mov $0,$$81
  f21 $0,104200 ; Group the objects into 3 bins based on mass: small=1, medium=2, big=3.
  mov $1,1
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "1d_recolor_cnt_37-1.asm"
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: IncrementSourceValueWhereTypeIsConstant
