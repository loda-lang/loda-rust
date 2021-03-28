; A066343 Beatty sequence for log_2(10).
; 3,6,9,13,16,19,23,26,29,33,36,39,43,46,49,53,56,59,63,66,69,73,76,79,83,86,89,93,96,99,MISMATCH
; 30 terms correct.

mod $1,4
mul $0,10
div $0,3
gcd $4,2
pow $1,4
trn $1,$2
gcd $1,3
add $1,$0
