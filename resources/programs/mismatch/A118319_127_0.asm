; 1,3,2,7,4,6,5,15,8,10,9,14,11,13,12,31,16,18,17,22,19,21,20,30,23,25,24,29,26,28,27,63,32,34,33,38,35,37,36,46

add $0,1
mov $1,270
mov $4,$0
gcd $4,64
pow $5,7
lpb $1
  mov $2,2
  sub $2,$0
  lpb $2
    mov $1,0
    mov $2,0
  lpe
  mov $2,$0
  mul $2,10
  lpb $2
    mod $2,2
    mov $3,1
    lpb $2
      mul $0,3
      add $4,1
      sub $2,1
    lpe
  lpe
  lpb $3
    div $0,2
    sub $3,1
    add $2,3
    add $4,$0
  lpe
  sub $1,1
lpe
mov $0,$4
