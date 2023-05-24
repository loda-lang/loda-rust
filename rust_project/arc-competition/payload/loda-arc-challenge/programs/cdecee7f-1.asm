; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100 ; address of vector[0].InputImage
mov $82,102 ; address of vector[0].ComputedOutputImage
mov $83,103 ; address of vector[0].PredictedOutputWidth
mov $84,104 ; address of vector[0].PredictedOutputHeight
mov $85,114 ; address of vector[0].InputMostPopularColor
lps $80
  mov $0,$$81 ; input image
  mov $1,$$85 ; most popular color across inputs

  ; rotate input by 90 degrees clockwise
  mov $3,$0
  mov $4,1
  f21 $3,101170 ; rotate cw
  ; $3 is the rotated input image

  ; extract mask
  mov $5,$3
  mov $6,$1
  f21 $5,101251 ; where color is different than background color
  ; $5 is the mask

  ; collect pixels
  mov $7,$3 ; the rotated input image
  mov $8,$5 ; mask
  f21 $7,102230 ; collect pixels
  ; $7 is a single row with the collected pixels

  ; change layout of the pixels
  mov $9,$7 ; pixels to be re-layouted
  mov $10,$$83 ; width = predicted width
  mov $11,$$84 ; height = predicted height
  mov $12,$1 ; background = most popular color
  f41 $9,102241 ; layout pixels with ReverseOddRows

  mov $0,$9

  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
  add $84,100
  add $85,100
lpe
