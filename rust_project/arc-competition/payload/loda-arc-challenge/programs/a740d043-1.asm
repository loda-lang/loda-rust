; ARC:a740d043
; Submitted by Simon Strandgaard
; Program Type: advanced

mov $40,0 ; an empty initial histogram image

; process "train" vector
mov $80,$97 ; set iteration counter = length of "train" vector
mov $81,100 ; address of first training data train[0].input
mov $82,101 ; address of first training data train[0].output
lps $80
  mov $0,$$81 ; load train[x].input image
  mov $1,$$82 ; load train[x].output image

  mov $10,$0
  f11 $10,101060 ; most popular color
  
  mov $9,$0
  f21 $9,101250 ; create mask where color is the most popular color
  
  f11 $9,101160 ; trim
  
  mov $8,$1
  f21 $8,101231 ; histogram with mask

  mov $9,1
  f21 $8,101221 ; get bottom row, the colors

  mov $9,$40
  f21 $8,101030 ; hstack
  mov $40,$8

  ; next iteration
  add $81,100 ; jump to address of next training input image
  add $82,100 ; jump to address of next training output image
lpe

; $40 is histogram image
f11 $40,101060 ; most popular color
; $40 is the most popular color in the histogram

; process "train"+"test" vectors
mov $80,$99 ; set iteration counter = length of "train"+"test" vectors
mov $81,100 ; address of vector[0].input
mov $82,102 ; address of vector[0].computed_output
lps $80
  mov $0,$$81 ; load vector[x].input image

  ; determine background color
  mov $10,$0
  f11 $10,101060 ; most popular color

  f11 $0,101160 ; trim

  ; replace background color with the output background color
  mov $1,$10 ; replace source color
  mov $2,$40 ; replace destination color
  f31 $0,101050 ; replace color

  mov $$82,$0 ; save vector[x].computed_output image

  ; next iteration
  add $81,100 ; jump to address of next input image
  add $82,100 ; jump to address of next computed_output image
lpe
