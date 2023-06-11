; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,114 ; address of vector[0].InputMostPopularColor
lps $80
  mov $0,$$81 ; input image
  mov $20,$$83 ; most popular color across inputs

  mov $10,$0
  f11 $10,101000 ; Get width of image
  div $10,2

  ; extract the separator
  mov $15,$0
  mov $16,$10
  f21 $15,101226 ; remove N left columns
  f21 $15,101227 ; remove N right columns
  ; $15 now holds the separator

  mov $5,$0
  mov $6,1 ; spacing
  f22 $5,102260 ; split into 2 columns
  ; $5..$6 are the 2 columns

  ; count the number of pixels in the left half of the image
  mov $30,$6
  mov $31,$20
  f21 $30,101251 ; where color is different than the most popular color
  ; $30 = mask
  mov $32,$30
  f11 $32,101244 ; count the number of ones
  ; $32 is the number of pixels in the left half

  ; count the number of pixels in the right half of the image
  mov $40,$5
  mov $41,$20
  f21 $40,101251 ; where color is different than the most popular color
  ; $40 = mask
  mov $42,$40
  f11 $42,101244 ; count the number of ones
  ; $42 is the number of pixels in the right half

  ; find the minimum value
  mov $50,$32
  min $50,$42
  ; $50 is the minimum value

  ; find the maximum value
  mov $51,$32
  max $51,$42
  ; $51 is the maximum value

  ; identify the register that holds the left image
  mov $60,$50
  cmp $60,$32
  add $60,5

  ; identify the register that holds the right image
  mov $61,$50
  cmp $61,$32
  mul $61,-1
  add $61,6

  ; recombine the images, ordered with the lowest pixels to the left, and the highest to the right
  mov $0,$$60 ; left
  mov $1,$15 ; separator
  mov $2,$$61 ; right
  f31 $0,101030 ; hstack

  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
