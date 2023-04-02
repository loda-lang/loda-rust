; ARC:b91ae062
; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,101000 ; get width
mov $2,5
f20 $1,1033 ; Assert input[0] is less than or equal to input[1].

mov $2,$0
f11 $2,101001 ; get height
mov $3,5
f20 $2,1033 ; Assert input[0] is less than or equal to input[1].

mov $5,$0
f11 $5,101240 ; number of unique colors
sub $5,1

mul $1,$5
mul $2,$5

f31 $0,101200 ; resize image
