; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,101000 ; get width
mod $1,2 ; spacing between the columns

f22 $0,102260 ; split into 2 columns
; $0..$1 are the 2 columns

f21 $0,101256 ; or
