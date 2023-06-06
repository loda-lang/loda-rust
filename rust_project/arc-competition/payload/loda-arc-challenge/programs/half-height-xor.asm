; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,101001 ; get height
mod $1,2 ; spacing between the columns

f22 $0,102261 ; split into 2 rows
; $0..$1 are the 2 rows

f21 $0,101254 ; xor
