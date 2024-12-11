; Submitted by Simon Strandgaard
; Program Type: simple

mov $5,$0
f11 $5,101000 ; get width
div $5,2

mov $4,$0
f21 $4,101222 ; get N left columns

mov $1,$5
f21 $0,101223 ; get N right columns

mov $1,$4
f21 $0,101256 ; or
