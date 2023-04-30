; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,0
f21 $0,101161 ; Image: Trim border with color to be trimmed

mov $1,$0
f11 $1,101000 ; Get width of image
mul $1,2

mov $2,$0
f11 $2,101001 ; Get height of image
mul $2,2

f31 $0,101200 ; Resize image
