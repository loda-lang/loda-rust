; A007887: a(n) = Fibonacci(n) mod 9.
; 0,1,1,2,3,5,8,4,3,7,1,8,0,8,8,7,6,4,1,5,6,2,8,1,0,1,1,2,3,5,8,4,3,7,1,8,0,8,8,7,6,4,1,5,6,2,8,1,0,1,1,2,3,5,8,4,3,7,1,8,0,8,8,7,6,4,1,5,6,2,8,1,0,1,1,2,3,5,8,4,3
; This program doesn't satisfy the requirement that the first 250 terms can be computed in 10 mio cycles.

mov $1,$0
cal $1,45
mod $1,9
