; A065883 Remove factors of 4 from n (i.e., write n in base 4, drop final zeros, then rewrite in decimal).
; 1,2,3,1,5,6,7,2,9,10,11,3,13,14,15,1,17,18,19,5,21,22,23,6,25,26,27,7,29,30,31,2,33,34,35,9,37,38,sni,15,61,62,63,MISMATCH
; 63 terms correct.

add $0,1
add $4,4
mov $3,11
dif $0,$4
log $3,4
dif $0,$4
mov $1,$0
