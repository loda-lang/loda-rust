; ARC:2013d3e2
; Submitted by Simon Strandgaard
; Program Type: simple

f11 $0,101160 ; trim
mov $4,$0
mov $5,$0
f11 $4,101000 ; get width
f11 $5,101001 ; get height
div $4,2
div $5,2
mov $1,$4
f21 $0,101220 ; get top rows
mov $1,$5
f21 $0,101222 ; get left columns
