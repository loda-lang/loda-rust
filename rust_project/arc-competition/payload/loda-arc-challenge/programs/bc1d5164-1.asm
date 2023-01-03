; ARC:bc1d5164
; Submitted by Simon Strandgaard
; Program Type: simple

mov $20,$0
f11 $20,101060 ; most popular color

mov $5,$0
mov $6,3
f21 $5,101220 ; top rows
f21 $5,101222 ; left columns
; $5 is the top/left corner

mov $6,$0
mov $7,3
f21 $6,101220 ; top rows
f21 $6,101223 ; right columns
; $6 is the top/right corner

mov $7,$0
mov $8,3
f21 $7,101221 ; bottom rows
f21 $7,101222 ; left columns
; $7 is the bottom/left corner

mov $8,$0
mov $9,3
f21 $8,101221 ; bottom rows
f21 $8,101223 ; right columns
; $8 is the bottom/right corner

; overlay the images, use the background color as the mask
mov $0,$5 ; top/left corner
mov $1,$6 ; top/right corner
mov $2,$20
f31 $0,101150 ; overlay image with color mask
; $0 is top/left corner overlayed with top/right corner

mov $1,$7 ; bottom/left corner
mov $2,$20
f31 $0,101150 ; overlay image with color mask
; $0 is top/left corner overlayed with top/right corner overlayed with bottom/left corner

mov $1,$8 ; bottom/right corner
mov $2,$20
f31 $0,101150 ; overlay image with color mask
; $0 is all the corners overlayed with each other
