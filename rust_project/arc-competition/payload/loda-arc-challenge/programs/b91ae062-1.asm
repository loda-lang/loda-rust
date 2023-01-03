; ARC:b91ae062
; Submitted by Simon Strandgaard
; Program Type: simple

mov $5,$0
f11 $5,101240 ; number of unique colors
sub $5,1

mov $1,$0
f11 $1,101000 ; get width

mov $2,$0
f11 $2,101001 ; get height

mul $1,$5
mul $2,$5

f31 $0,101200 ; resize image
