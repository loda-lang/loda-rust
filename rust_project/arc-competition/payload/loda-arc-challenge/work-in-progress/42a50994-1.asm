; ARC:42a50994
; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,101060 ; most popular color

; $1 is the background_color
f21 $0,101090 ; denoise type 4. TODO: I have no denoise function of this type.
