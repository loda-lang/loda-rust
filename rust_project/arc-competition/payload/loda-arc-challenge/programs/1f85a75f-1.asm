; Submitted by Simon Strandgaard
; Program Type: simple

mov $3,$0
f11 $3,101060 ; most popular color
; $3 is background_color

; remove noisy pixels
mov $4,$0
mov $5,3 ; number of noise colors to remove
f21 $4,101092 ; denoise type 3
;mov $0,$4

; mask
mov $6,$4 ; image
mov $7,$3 ; color
f21 $6,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.
;mov $0,$6

; multiply input image by mask
mov $8,$6
mov $9,$0
mov $10,$3
f31 $8,102130 ; Pick pixels from one image.
mov $0,$8

; remove space around the object
f11 $0,101160 ; trim
