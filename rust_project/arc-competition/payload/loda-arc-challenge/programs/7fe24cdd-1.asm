; ARC:7fe24cdd
; Submitted by Simon Strandgaard
; Program Type: simple

mov $5,$0 ; original corner

; construct top half
mov $1,$0
mov $2,1
f21 $1,101170 ; rotate cw
f21 $0,101030 ; hstack
; $0 is top half

; construct bottom half
mov $6,2
f21 $5,101170 ; rotate cw cw
mov $1,$5
mov $2,1
f21 $1,101170 ; rotate cw
mov $2,$5
f21 $1,101030 ; hstack
; $1 is bottom half

; join top half and bottom half
f21 $0,101040 ; vstack
