mov $10,$0
add $0,100

mov $1,$0
mov $2,$0
mov $3,$0

mov $6,1
; ACT-BEGIN
clr $$6,2 ; Same as clr $1,2
; ACT-END

mov $0,$$10
