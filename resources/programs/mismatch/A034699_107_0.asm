; 1,2,3,4,5,3,7,8,9,5,11,4,13,7,5,16,17,9,19,5,7,11,23,8,25,13,27,7,29,5,31,32,11,17,7,9,37,19,13,8

mov $1,1
mov $2,1
add $0,1
lpb $0
  mov $3,$0
  lpb $3
    mov $4,$0
    mod $4,$2
    cmp $4,0
    cmp $4,0
    mov $5,$2
    cmp $5,1
    add $2,1
    max $4,$5
    sub $3,$4
  lpe
  mul $3,5
  mov $3,$2
  sub $3,1
  add $3,1
  mov $5,2
  lpb $0
    dif $0,$2
    mul $5,$3
  lpe
  div $1,2
  sub $4,1
  add $4,2
  dif $5,$3
  div $5,$4
  mul $2,$5
lpe
mov $0,$1
mov $0,$2
