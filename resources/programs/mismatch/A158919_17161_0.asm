; 0,1,3,5,7,9,11,12,14,16,18,20,22,23,25,27,29,31,33,34,36,38,40,42,44,45,47,49,51,53,55,57,58,60,62,64,66,68,69,71

mul $0,1236
div $0,672
mov $2,$0
lpb $0
  lpb $1
    sub $1,4
    pow $1,3
  lpe
  mul $1,2
  seq $1,196
  seq $1,54519
  mov $0,$1
lpe
add $0,1
mov $0,$2