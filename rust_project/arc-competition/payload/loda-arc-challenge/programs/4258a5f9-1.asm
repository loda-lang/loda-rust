; ARC:4258a5f9
; Submitted by Simon Strandgaard
; Program Type: advanced

mov $40,0 ; outline color

; process "train" vector
mov $80,$97 ; set iteration counter = length of "train" vector
mov $81,100 ; address of first training data train[0].input
mov $82,101 ; address of first training data train[0].output
lps $80
  mov $0,$$81 ; load train[x].input image
  mov $1,$$82 ; load train[x].output image

  ; analyze the output images
  f12 $1,101070 ; least popular colors
  mov $40,$2 ; get the outline color

  ; next iteration
  add $81,100 ; jump to address of next training input image
  add $82,100 ; jump to address of next training output image
lpe

; process "train"+"test" vectors
mov $80,$99 ; set iteration counter = length of "train"+"test" vectors
mov $81,100 ; address of vector[0].input
mov $82,102 ; address of vector[0].computed_output
lps $80
  mov $0,$$81 ; load vector[x].input image

  mov $5,$0
  f11 $5,101060 ; most popular color

  mov $1,$40 ; outline color
  mov $2,$5 ; background color
  f31 $0,101080 ; draw outline

  mov $$82,$0 ; save vector[x].computed_output image

  ; next iteration
  add $81,100 ; jump to address of next input image
  add $82,100 ; jump to address of next computed_output image
lpe
