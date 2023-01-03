; ARC:007bbfb7
; Submitted by Simon Strandgaard
; Program Type: simple

mov $22,0 ; background color
mov $19,$0

mov $20,$0
f11 $20,101000 ; get width

mov $21,$0
f11 $21,101000 ; get height

; div by zero if width is != 3
mov $33,$20
cmp $33,3
mov $34,1
div $34,$33

; div by zero if height is != 3
mov $33,$21
cmp $33,3
mov $34,1
div $34,$33

mov $0,$20
pow $0,2 ; width * width
mov $1,$21
pow $1,2 ; height * height
mov $2,0 ; fill color
f31 $0,101010 ; create image of size 9x9 with color 0

mov $11,0 ; reset y position

mov $8,$21 ; height
lps $8 ; loop over rows

  mov $10,0 ; reset x position

  mov $9,$20 ; width
  lps $9 ; loop over columns

    mov $3,$19 ; input image
    mov $4,$10 ; x
    mov $5,$11 ; y
    f31 $3,101002 ; get pixel

    cmp $3,$22
    cmp $3,0
    ; if the pixel is different than the background color
    ; then overlay the image
    lps $3

      mov $1,$19 ; the image to be overlayed

      mov $2,$10 ; x
      mul $2,$20 ; x * width

      mov $3,$11 ; y
      mul $3,$21 ; y * width

      f41 $0,101151 ; overlay with image
    lpe

    add $10,1 ; next column
  lpe

  add $11,1 ; next row
lpe
