; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99 ; set iteration counter = length of "train"+"test" vectors
mov $82,102 ; address of vector[0].computed_output
mov $83,103 ; address of vector[0].predicted_output_width
mov $84,104 ; address of vector[0].predicted_output_height
lps $80
    mov $0,$$83 ; predicted output width
    mov $1,$$84 ; predicted output height
    mov $2,1 ; fill color
    f31 $0,101010 ; Create new image with size (x, y) and filled with color z

    mov $$82,$0 ; save vector[x].computed_output image

    ; next iteration
    add $82,100 ; jump to address of next computed_output image
    add $83,100 ; jump to address of next predicted_output_width
    add $84,100 ; jump to address of next predicted_output_height
lpe
