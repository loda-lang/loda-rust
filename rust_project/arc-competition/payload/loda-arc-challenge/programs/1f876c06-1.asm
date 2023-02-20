; ARC:1f876c06
; Submitted by Simon Strandgaard
; Program Type: simple

mov $20,255 ; color when there is no neighbour

mov $21,$0
f11 $21,101000 ; get width

mov $22,$0
f11 $22,101001 ; get height

; ignore mask
mov $1,$0
mov $2,$0
f11 $2,101060 ; most popular color
f21 $1,101250 ; mask where color is
; $2 is most popular color
; $1 is the ignore mask

; neighbour_up_left
mov $10,$0
mov $11,$1
mov $12,$20
f31 $10,102064 ; neighbour 'UpLeft'
mov $3,$10

; neighbour_up_right
mov $10,$0
mov $11,$1
mov $13,$20
f31 $10,102065 ; neighbour 'UpRight'
mov $4,$10

; neighbour_down_left
mov $10,$0
mov $11,$1
mov $13,$20
f31 $10,102066 ; neighbour 'DownLeft'
mov $5,$10

; neighbour_down_right
mov $10,$0
mov $11,$1
mov $13,$20
f31 $10,102067 ; neighbour 'DownRight'
mov $6,$10

mov $14,$0 ; clone input image

mov $41,0 ; reset y position

mov $8,$22 ; height
lps $8 ; loop over rows
  mov $40,0 ; reset x position
  mov $9,$21 ; width
  lps $9 ; loop over columns

    ; color of 'UpLeft'
    mov $10,$3 ; neighbour_up_left
    mov $11,$40 ; x
    mov $12,$41 ; y
    f31 $10,101002 ; get pixel
    mov $30,$10

    ; color of 'UpRight'
    mov $10,$4 ; neighbour_up_right
    mov $11,$40 ; x
    mov $12,$41 ; y
    f31 $10,101002 ; get pixel
    mov $31,$10

    ; color of 'DownLeft'
    mov $10,$5 ; neighbour_down_left
    mov $11,$40 ; x
    mov $12,$41 ; y
    f31 $10,101002 ; get pixel
    mov $32,$10

    ; color of 'DownRight'
    mov $10,$6 ; neighbour_down_right
    mov $11,$40 ; x
    mov $12,$41 ; y
    f31 $10,101002 ; get pixel
    mov $33,$10

    ; $30 = color_up_left
    ; $31 = color_up_right
    ; $32 = color_down_left
    ; $33 = color_down_right

    ;if color_down_left == color_up_right && color_down_left != color_when_there_is_no_neighbour {
    ;    let _ = result_image.set(x, y, color_down_left);
    ;}
    mov $18,$32
    cmp $18,$31
    mov $19,$32
    cmp $19,$20
    cmp $19,0
    mul $19,$18
    lps $19
      mov $15,$40 ; x
      mov $16,$41 ; y
      mov $17,$32
      f41 $14,101003 ; set pixel
    lpe
    mov $17,$32

    ;if color_down_right == color_up_left && color_down_right != color_when_there_is_no_neighbour {
    ;    let _ = result_image.set(x, y, color_down_right);
    ;}
    mov $18,$33
    cmp $18,$30
    mov $19,$33
    cmp $19,$20
    cmp $19,0
    mul $19,$18
    lps $19
      mov $15,$40 ; x
      mov $16,$41 ; y
      mov $17,$30
      f41 $14,101003 ; set pixel
    lpe

    add $40,1 ; next column
  lpe
  add $41,1 ; next row
lpe

mov $0,$14
