; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $33,$0
  f11 $33,101000 ; Get width of image
  div $33,2
  mov $30,$0
  f11 $30,101001 ; Get height of image
  div $30,2
  mov $20,$0
  mov $21,$82
  mov $22,$30
  f31 $20,101180 ; Adjust image offset(dx, dy) with wrap
  mov $21,1
  f21 $20,101221 ; get N bottom rows
  mov $21,$33
  f21 $20,101222 ; get N left columns
  mov $11,$20
  mov $12,1
  mov $13,0
  f31 $11,101180 ; Adjust image offset(dx, dy) with wrap
  mov $10,$20
  f21 $10,101040 ; Image.vstack. vertical stack of 2 images
  mov $1,$10
  f21 $0,101052 ; Image: replace colors with palette image
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "bda2d7a6-2.asm"
; mutate: ReplaceLineWithHistogram
; mutate: SetSourceToDirect
