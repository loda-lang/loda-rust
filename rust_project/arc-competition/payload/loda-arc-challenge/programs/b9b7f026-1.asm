; ARC:b9b7f026
; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,101060 ; most popular color
; $1 is background color
f21 $0,101110 ; detect holes

mov $2,$0
f11 $2,101070 ; least popular color
; $2 is the corner color

mov $0,1 ; width=1
mov $1,1 ; height=1
f31 $0,101010 ; create image with color
