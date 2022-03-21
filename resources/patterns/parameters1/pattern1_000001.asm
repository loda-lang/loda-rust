; PATTERN-NAME: Greatest k such that PARAMETER0^k divides n.

add $0,1
lpb $0
  dif $0,2 ; source=parameter 0
  add $1,1
lpe
mov $0,$1

; A007814: 2
; A007949: 3
; A112765: 5
; A122840: 10
; A122841: 6
; A214411: 7
; A235127: 4
; A244413: 8
