; 0,4,8,16,20,24,32,36,40,48,52,56,64,68,72,80,84,88,96,100,104,112,116,120,128,132,136,144,148,152,160,164,168,176,180,184,192,196,200,208

mov $2,$0
mov $1,$2
mul $0,2
lpb $0
  add $2,1
  sub $0,6
  mov $1,$2
lpe
mov $4,$7
mov $6,$7
lpb $4
  sub $4,2
  add $1,$0
lpe
mov $2,$3
mov $6,$5
mov $5,0
lpb $4
  sub $4,1
  add $5,$6
lpe
mov $3,0
mov $6,$5
lpb $3
  add $1,$6
  sub $3,1
lpe
mov $4,$3
mov $5,0
lpb $4
  sub $1,1
  add $5,$6
lpe
mov $0,$2
mov $6,$5
mov $5,0
lpb $4
  sub $4,1
  add $0,$2
lpe
mov $2,$4
mov $0,$1
mov $4,0
lpb $4
  sub $3,1
  add $0,2
lpe
mov $3,2
mov $6,$5
lpb $3
  add $1,$1
  sub $3,1
lpe
mov $0,$1