; ARC:3af2c5a8
; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,101190 ; flip x
f21 $0,101030 ; hstack
mov $1,$0
f11 $1,101191 ; flip y
f21 $0,101040 ; vstack
