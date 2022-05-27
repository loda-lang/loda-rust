; A067497 Smallest power of 2 with n+1 digits (n>=0). Also numbers k such that 1 is the first digit of 2^k.
; 0,4,7,10,14,17,20,24,27,30,34,37,40,44,47,50,54,57,60,64,67,70,74,77,80,84,87,90,94,97,100,MISMATCH
; 31 terms correct.

mul $0,10
add $0,2
trn $1,2
add $1,$0
gcd $2,2
gcd $4,6
div $1,3
mov $0,$1
