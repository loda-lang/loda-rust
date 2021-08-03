; A059537: Beatty sequence for zeta(3).
; 1,2,3,4,6,7,8,9,10,12,13,14,15,16,18,19,20,21,22,24,25,26,27,28,30,31,snip,115,116,117,MISMATCH
; 99 terms correct.

seq $2,52126
add $4,1
add $3,1
mul $3,$2
dif $0,$2
mov $4,6
add $0,$3
mul $4,$0
sub $2,$0
gcd $1,$4
gcd $2,$0
div $1,5
