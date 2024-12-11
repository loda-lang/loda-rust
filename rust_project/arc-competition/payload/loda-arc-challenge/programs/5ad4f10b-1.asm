; ARC:5ad4f10b
; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
mov $2,$0
mov $3,$0
mov $9,$0

f11 $3,101060 ; most popular color
; $3 is background_color

mov $5,$0 ; noisy image
mov $6,$3 ; background_color
f21 $5,101090 ; denoise type 1
; $5 is denoised image

; $9 is noisy image
mov $10,$5 ; denoised image
f21 $9,101100 ; extract 1 noise color
; $9 is the most popular noise color

mov $12,$5 ; denoised image
f11 $12,101160 ; trim
f11 $12,101140 ; remove duplicates

mov $0,$12
mov $1,$3 ; background color
mov $2,$9 ; noise color
f31 $0,101051 ; replace colors other than
