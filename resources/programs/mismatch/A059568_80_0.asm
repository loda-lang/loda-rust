; 3,7,11,14,18,22,26,29,33,37,41,44,48,52,55,59,63,67,70,74,78,82,85,89,93,96,100,104,108,111,115,119,123,126,130,134,137,141,145,149

mov $2,$0
add $2,1
add $0,$2
add $0,$2
mul $0,10
mov $1,$2
mov $2,10
mov $3,2
lpb $0
  sub $0,1
  add $1,1
  mov $3,$2
  trn $0,$3
lpe
mov $0,$1