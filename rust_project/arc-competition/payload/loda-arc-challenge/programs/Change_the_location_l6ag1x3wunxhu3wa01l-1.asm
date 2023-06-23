; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,114
lps $80
  mov $0,$$81
  mov $20,$$83
  mov $10,$0
  f11 $10,101000 ; Get width of image
  div $10,2
  mov $15,$0
  mov $16,$10
  f21 $15,101226 ; remove N left columns
  f21 $15,101227 ; remove N right columns
  mov $5,$0
  mov $6,1
  f22 $5,102260 ; Split image into 2 columns with same size
  mov $30,$6
  mov $31,$20
  f21 $30,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.
  mov $33,$30
  f11 $32,101244 ; Number of ones in image.
  mov $40,$5
  mov $41,$20
  f21 $40,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.
  mov $42,$40
  f11 $42,101244 ; Number of ones in image.
  mov $50,$32
  min $50,$42
  mov $51,$32
  max $51,$42
  mov $60,$50
  cmp $60,$32
  add $60,5
  mov $61,$50
  cmp $61,$32
  mul $61,-1
  add $61,6
  mov $0,$$60
  mov $1,$15
  mov $2,$$61
  f31 $0,101030 ; Image.hstack. horizontal stack of 3 images
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe

; template: "Order7.asm"
; mutate: ReplaceSourceWithHistogram, no change
; mutate: IncrementTargetValueWhereTypeIsDirect
