mov $8,100
mov $100,5
mov $101,4
mov $102,3
mov $103,2
mov $104,1
lpb $$8
  add $1,1
  add $8,1
lpe
mov $0,$1
