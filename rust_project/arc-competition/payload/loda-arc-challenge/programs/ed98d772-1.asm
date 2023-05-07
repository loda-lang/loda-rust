; Submitted by Simon Strandgaard
; Program Type: simple

mov $3,$0
mov $1,$0

mov $2,2
f21 $1,101170 ; Image: Rotate by x * 90 degrees
f21 $0,101040 ; vstack
; $0 holds the left side of the result

mov $4,-1
f21 $3,101170 ; Image: Rotate by x * 90 degrees
mov $4,$3
mov $5,2
f21 $4,101170 ; Image: Rotate by x * 90 degrees
f21 $3,101040 ; vstack
; $3 holds the right side of the result

mov $1,$3
f21 $0,101030 ; hstack
