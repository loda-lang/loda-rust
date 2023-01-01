; ARC:bda2d7a6
; Submitted by Simon Strandgaard
; Program Type: simple

mov $33,$0
f11 $33,101000 ; get width of image
div $33,2 ; $33 is half width

mov $30,$0
f11 $30,101001 ; get height of image
div $30,2 ; $30 is half height

; move the mid-row to the bottom
mov $20,$0
mov $21,0
mov $22,$30
f31 $20,101180 ; offset

; take just the mid row
mov $21,1
f21 $20,101221 ; take bottom row

; take half-width of the pixels
mov $21,$33
f21 $20,101222 ; take left half of image
; $20 is mid row, left half of input image

mov $11,$20
mov $12,1
mov $13,0
f31 $11,101180 ; offset, cycles the palette by 1 pixel

mov $10,$20
f21 $10,101040 ; vstack
; $10 is a 2xN image, the top row is the source colors, the bottom row is the target colors

; replace colors of the image using the palette image
mov $1,$10 ; palette image
f21 $0,101052 ; replace colors using palette image
