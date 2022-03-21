; PATTERN-NAME: ?

mov $1,2 ; source=parameter 0
mov $3,3 ; source=parameter 1
lpb $0
  sub $0,1
  mov $2,$3
  mul $3,3 ; source=parameter 2
  add $3,$1
  mov $1,$2
lpe
mov $0,$1 ; source=parameter 3

; A006497: 2,3,3,$1
; A015453: 1,1,7,$1
; A048697: 8,1,2,$1
; A048875: 2,1,4,$3
; A048876: 3,1,4,$3
; A048877: 4,1,4,$3
; A048879: 6,1,4,$3
; A097924: 2,7,4,$1
; A108300: 2,1,3,$3
; A109109: -6,1,10,$3
