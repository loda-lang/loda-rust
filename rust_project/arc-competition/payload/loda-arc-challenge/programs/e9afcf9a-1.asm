; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
lps $80
  mov $10,$$81 ; input image
  f11 $10,102206 ; sort columns by pixel value

  mov $11,$10
  f11 $11,101191 ; flip y

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

  mov $1,$10
  mov $2,$11
  f31 $0,102132 ; Pick pixels from two images. When the mask is 0 then pick `image_a`. When the mask is [1..255] then pick from `image_b`.

  mov $$82,$0
  add $81,100
  add $82,100
lpe
