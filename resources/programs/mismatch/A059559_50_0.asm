; 1,3,4,6,7,9,10,12,13,15,17,18,20,21,23,24,26,27,29,30,32,34,35,37,38,40,41,43,44,46,48,49,51,52,54,55,57,58,60,61

mov $2,0
add $2,1
mov $6,$0
lpb $2
  mov $0,$6
  sub $2,1
  sub $0,$2
  mov $5,$0
  mov $10,2
  lpb $10
    add $0,$10
    sub $10,1
    sub $0,1
    mov $9,13
    mul $9,$0
    sub $9,$0
    mov $3,$9
    mul $3,4
    div $3,31
    mov $4,$3
    mov $7,$10
    lpb $2
      sub $7,1
      mov $8,$4
    lpe
  lpe
  lpb $5
    mov $0,10
    sub $8,$4
  lpe
  mov $4,$8
  mul $4,2
  add $4,1
  add $1,$4
lpe
mov $1,$0
mov $0,$3