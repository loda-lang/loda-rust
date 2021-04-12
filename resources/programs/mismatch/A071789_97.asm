; A071789: Decimal expansion of the first (of 10) decimal selvage numbers; the n-th digit of a decimal selvage number, x, is equal to the tenths digit of n*x.
; 1,2,3,4,6,7,8,9,1,2,3,4,6,7,8,9,0,2,3,4,5,7,8,9,0,2,3,4,5,7,8,9,0,1,3,4,5,6,8,9,snip,7,8,9,MISMATCH
; 97 terms correct.

mov $1,$0
cal $1,115180 ; Beatty sequence for (Champernowne constant)*10 = 1.234567891011121314....
mod $1,10
