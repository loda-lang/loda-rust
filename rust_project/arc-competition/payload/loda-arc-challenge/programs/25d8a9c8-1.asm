; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,101000 ; get width
; $1 is the width of the input image

f11 $0,101241 ; count unique colors per row

; $1 is the width of the input image
mov $2,1
f31 $0,102120 ; repeat image
