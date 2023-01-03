; ARC:9172f3a0
; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
mov $2,$0
f11 $1,101000 ; get width
f11 $2,101001 ; get height
mul $1,3
mul $2,3
; $1 is width
; $2 is height
f31 $0,101200 ; resize image
