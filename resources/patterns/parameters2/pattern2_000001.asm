; PATTERN-NAME: Expansion of x/(1 - PARAMETER1*x - PARAMETER0*x^2).

mov $1,1
lpb $0
  sub $0,1
  mov $2,$3
  mul $2,2 ; source=parameter 0
  mul $3,3 ; source=parameter 1
  add $3,$1
  mov $1,$2
lpe
mov $0,$3

; A007482: 2,3
; A015530: 3,4
; A015535: 2,5
; A015536: 3,5
; A015537: 4,5
; A015544: 8,5
; A015548: 12,5
; A015555: 2,7
; A015559: 3,7
; A015561: 4,7
; A015568: 10,7
; A015574: 3,8
; A015575: 5,8
; A015579: 2,9
; A015580: 4,9
; A015588: 3,10
; A015589: 7,10
; A015591: 9,10
; A015593: 2,11
; A015598: 6,11
; A015602: 8,11
; A015603: 9,11
