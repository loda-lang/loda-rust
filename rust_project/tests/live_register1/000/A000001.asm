mov $2,$0
mul $0,0  ; clear $0
; at this point $0 is dead and $2 live.
mov $0,$2
mov $2,0  ; clear $2
; at this point only $0 is live