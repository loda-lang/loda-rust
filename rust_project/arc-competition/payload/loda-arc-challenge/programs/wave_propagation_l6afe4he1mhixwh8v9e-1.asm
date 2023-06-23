; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $10,$0
  f11 $10,101000 ; Get width of image
  div $10,2
  mov $11,8
  f20 $10,1033 ; Assert input[0] is less than or equal to input[1].
  mov $2,0
  mov $7,0
  lps $10
    mov $4,$7
    mov $5,0
    mov $3,$0
    f31 $3,101181 ; Adjust image offset(dx, dy) with clamp
    f21 $2,101040 ; Image.vstack. vertical stack of 2 images
    add $7,1
  lpe
  mov $0,$3
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "bbc9ae5d-1.asm"
; mutate: IncrementSourceValueWhereTypeIsDirect
