; ARC:7f4411dc
; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,101060 ; most popular color

; $0 is noisy image
; $1 is background_color
f21 $0,101090 ; denoise type 1
; $0 is denoised image
