; A333183: Number of digits in concatenation of first n positive even integers.
; Submitted by Simon Strandgaard
; 1,2,3,4,6,8,10,12,14,16,18,20,22,24,26,28,30,32,34,36,38,40,42,44,46,48,50,52,54,56,58,60,62,64,66,68,70,72,74,76,78,80,82,84,86,88,90,92,94,97,100,103,106,109,112,115,118,121,124,127,130,133,136,139,142,145,148,151,154

lpb $0
  sub $0,1
  add $4,$1
  add $4,3
  mov $5,$3
  add $5,$2
  gcd $1,$3
  mov $2,$3
  add $2,$4
  mov $3,$5
  sub $3,$1
lpe
gcd $0,$4
div $0,3
add $0,1
