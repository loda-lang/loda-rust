; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,101000 ; Get width of image

mov $2,$0
f11 $2,101001 ; Get height of image

; $1 is count x = width of the image
; $2 is count y = height of the image
f31 $0,102120 ; Make a big image by repeating the current image.
