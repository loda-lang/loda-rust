; A006450: Prime-indexed primes: primes with prime subscripts.
; Coded manually 2021-03-28 by Simon Strandgaard, https://github.com/neoneye
; 3,5,11,17,31,41,59,67,83,109,127,157,179,191,211,241,277,283,331,353,367,401,431,461,509,547,563,587,599,617,709,739,773,797,859,877,919,967,991,1031,1063,1087,1153,1171,1201,1217,1297,1409,1433,1447,1471
; This program doesn't satisfy the requirement that the first 250 terms can be computed in 10 mio cycles.

mov $1,$0
cal $1,40
sub $1,1
cal $1,40
