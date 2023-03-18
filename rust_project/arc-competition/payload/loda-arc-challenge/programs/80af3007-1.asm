; Submitted by Simon Strandgaard
; Program Type: simple

mov $3,$0
f11 $3,101060 ; most popular color
; $3 is background_color

; remove space around the object
mov $5,$0
f11 $5,101160 ; trim

; scaled down to 3x3
mov $8,$5
mov $9,3
mov $10,3
f31 $8,101200 ; resize

; mask
mov $10,$8 ; image
mov $11,$3 ; color
f21 $10,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

; an empty tile
mov $14,$3 ; color
mov $13,3 ; height
mov $12,3 ; width
f31 $12,101010 ; Create new image with size (x, y) and filled with color z

; Layout tiles
mov $15,$10
mov $16,$12 ; tile0
mov $17,$8 ; tile1
f31 $15,102110 ; Create a big composition of tiles.
mov $0,$15
