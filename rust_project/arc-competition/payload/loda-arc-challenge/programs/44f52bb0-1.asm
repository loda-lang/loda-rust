; ARC:44f52bb0
; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,101190 ; flip x
cmp $0,$1
mov $2,1 ; color when there is symmetry
mul $2,$0
cmp $0,0
mul $0,7 ; color when there is no symmetry
add $2,$0
mov $0,1 ; output image width
mov $1,1 ; output image height
f31 $0,101010 ; create image
