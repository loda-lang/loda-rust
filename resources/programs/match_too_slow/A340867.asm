; A340867 a(n) = (prime(n) - a(n-1)) mod 4; a(0)=0.
; Coded manually 2021-03-28 by Simon Strandgaard, https://github.com/neoneye
; 0,2,1,0,3,0,1,0,3,0,1,2,3,2,1,2,3,0,1,2,1,0,3,0,1,0,1,2,1,0,1,2,1,0,3,2,1,0,3,0,1,2,3,0,1,0,3,0,3,0,1,0,3,2,1,0,3,2,1,0,1,2,3,0,3,2,3,0,1,2,3,2,1,2,3,0,3,2,3,2,3,0,1,2,3,0
; This program doesn't satisfy the requirement that the first 250 terms can be computed in 10 mio cycles.

mov $1,$0
cal $1,8347 ; a(n) = Sum_{i=0..n-1} (-1)^i * prime(n-i).
mod $1,4
