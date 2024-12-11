; Submitted by Simon Strandgaard
; Program Type: simple

; tile_width
mov $2,$0
f11 $2,101000 ; Get width of image
mov $3,5
f20 $2,1033 ; Assert input[0] is less than or equal to input[1].

; tile_height
mov $3,$0
f11 $3,101001 ; Get height of image
mov $4,5
f20 $3,1033 ; Assert input[0] is less than or equal to input[1].

; tile
mov $7,0 ; color
mov $6,$3 ; height
mov $5,$2 ; width
f31 $5,101010 ; Create new image with size (x, y) and filled with color z

; mask
mov $10,$0 ; image
mov $11,$1 ; color
f21 $10,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

mov $11,$5 ; tile0
mov $12,$0 ; tile1
f31 $10,102110 ; Create a big composition of tiles.

mov $0,$10
