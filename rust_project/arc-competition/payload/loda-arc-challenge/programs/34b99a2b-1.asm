; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
lps $80
  mov $0,$$81

  mov $8,$0
  f11 $8,101000 ; Get width of image
  sub $8,1
  div $8,2

  mov $1,$0
  mov $2,$8
  f21 $1,101222 ; get N left columns

  mov $2,$0
  mov $3,$8
  f21 $2,101223 ; get N right columns
  mov $0,$2

  f21 $0,101254 ; xor

  mov $$82,$0
  add $81,100
  add $82,100
lpe
