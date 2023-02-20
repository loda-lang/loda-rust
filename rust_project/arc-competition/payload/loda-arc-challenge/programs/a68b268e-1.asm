; ARC:a68b268e
; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,101060 ; most popular color

; W = compute (width-1) / 2
mov $2,$0
f11 $2,101000 ; Get width of image
sub $2,1
div $2,2

; H = compute (height-1) / 2
mov $3,$0
f11 $3,101001 ; Get height of image
sub $3,1
div $3,2

; top left corner of size WxH
mov $10,$0
mov $11,$3
f21 $10,101220 ; get N top rows
mov $11,$2
f21 $10,101222 ; get N left columns

; top right corner of size WxH
mov $15,$0
mov $16,$3
f21 $15,101220 ; get N top rows
mov $16,$2
f21 $15,101223 ; get N right columns

; bottom left corner of size WxH
mov $20,$0
mov $21,$3
f21 $20,101221 ; get N bottom rows
mov $21,$2
f21 $20,101222 ; get N left columns

; bottom right corner of size WxH
mov $25,$0
mov $26,$3
f21 $25,101221 ; get N bottom rows
mov $26,$2
f21 $25,101223 ; get N right columns

; zstack where the images are placed on top of each other
; zindex 0 - the bottom
mov $30,$25 ; bottom right

; zindex 1
mov $31,$20 ; bottom left
mov $32,$1 ; most popular color
f31 $30,101150 ; overlay image

; zindex 2
mov $31,$15 ; top right
mov $32,$1 ; most popular color
f31 $30,101150 ; overlay image

; zindex 3 - the top
mov $31,$10 ; top left
mov $32,$1 ; most popular color
f31 $30,101150 ; overlay image

mov $0,$30
