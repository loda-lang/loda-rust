; 1,2,3,4,6,7,8,10,11,12,14,15,16,18,19,20,21,22,23,25,26,27,29,30,31,33,34,35,36,37,38,40,41,42,44,45,46,48,49,50

mov $4,$0
mov $5,1
lpb $5
  mov $3,$0
  sub $5,1
  mov $6,2
  lpb $6
    mov $0,$3
    sub $0,1
    div $0,3
    mov $1,$0
    mov $2,$0
    sub $2,1
    div $2,4
    sub $1,$2
    sub $6,1
  lpe
lpe
add $1,1
add $1,$4
mov $0,$1