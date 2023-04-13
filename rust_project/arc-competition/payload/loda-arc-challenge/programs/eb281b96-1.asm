; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,101191 ; Image: flip y

mov $2,1 ; number of rows = 1
f21 $1,101224 ; remove top row

f21 $0,101040 ; vstack

mov $1,$0
f11 $1,101191 ; Image: flip y

mov $2,1 ; number of rows = 1
f21 $1,101224 ; remove top row

f21 $0,101040 ; vstack
