; ARC:bbc9ae5d
; Submitted by Simon Strandgaard
; Program Type: simple

mov $10,$0
f11 $10,101000 ; get image width
div $10,2
; $10 is the height of the final image

mov $2,0
mov $7,0
lps $10

  ; clone the input image, and offset it
  mov $4,$7
  mov $5,0
  mov $3,$0
  f31 $3,101181 ; offset clamp

  ; glue onto the bottom of the result image
  f21 $2,101040 ; vstack

  add $7,1
lpe
mov $0,$2
