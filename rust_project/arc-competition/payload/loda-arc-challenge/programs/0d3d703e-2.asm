; ARC:0d3d703e
; Submitted by Simon Strandgaard
; Program Type: advanced

mov $40,0 ; palette image accumulated

; process "train" vector
mov $80,$97 ; set iteration counter = length of "train" vector
mov $81,100 ; address of first training data train[0].input
mov $82,101 ; address of first training data train[0].output
lps $80
  mov $0,$$81 ; load train[x].input image
  mov $1,$$82 ; load train[x].output image

  ; analyze the images
  f21 $0,101132 ; build palette image with color mapping from input to output
  mov $41,$0
  f21 $40,101030 ; hstack of the palette images

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

  ; replace colors of the image using the palette image
  mov $1,$40 ; palette image
  f21 $0,101052 ; replace colors using palette image

  mov $$82,$0 ; save vector[x].computed_output image

  ; next iteration
  add $81,100 ; jump to address of next input image
  add $82,100 ; jump to address of next computed_output image
lpe
