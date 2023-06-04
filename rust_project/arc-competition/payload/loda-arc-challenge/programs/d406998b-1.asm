; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,114 ; address of vector[0].InputMostPopularColor
lps $80
  mov $10,$$81 ; input image
  mov $20,$$83 ; most popular color across inputs

  mov $15,$10
  mov $16,$20
  f21 $15,101251 ; where color is different than the most popular color
  ; $15 = mask

  mov $0,$10
  f11 $0,101000 ; width of image
  
  mov $1,$10
  f11 $1,101001 ; height of image

  ; $0 = width
  ; $1 = height
  mov $2,1 ; color0
  mov $3,1 ; count0
  mov $4,0 ; color1
  mov $5,1 ; count1
  f61 $0,101260 ; Alternating columns with two colors
  ; $0 = stripes that starts from the left edge
  
  f11 $0,101190 ; flip x
  ; $0 = stripes that starts from the right edge

  mov $1,$15
  f21 $0,101255 ; boolean AND
  ; $0 = intersection of the mask with the alternating columns

  mov $1,$10
  mov $2,255
  f31 $0,102131 ; Pick pixels from image and color. When the mask is 0 then pick from the image. When the mask is [1..255] then use the `default_color`.

  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
lpe
