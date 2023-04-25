; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99 ; set iteration counter = length of "train"+"test" vectors
mov $81,100 ; address of vector[0].input
mov $82,102 ; address of vector[0].computed_output
mov $83,103 ; address of vector[0].predicted_output_width
mov $84,104 ; address of vector[0].predicted_output_height
lps $80
    mov $0,$$81 ; load vector[x].input image

    mov $1,$$83 ; predicted output width
    mov $2,$$84 ; predicted output height
    f31 $0,101200 ; Resize image to size width x height

    mov $$82,$0 ; save vector[x].computed_output image

    ; next iteration
    add $81,100 ; jump to address of next input image
    add $82,100 ; jump to address of next computed_output image
    add $83,100 ; jump to address of next predicted_output_width
    add $84,100 ; jump to address of next predicted_output_height
lpe
