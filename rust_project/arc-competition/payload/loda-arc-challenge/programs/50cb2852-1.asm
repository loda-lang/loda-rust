; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,102140 ; Traverse all pixels in the 3x3 convolution and count how many have the same color as the center.
mov $2,8
f21 $1,101253 ; Convert to a mask image by converting `pixel_color >= threshold_color` to 1 and converting anything else to to 0.

mov $2,$0
mov $3,0
f21 $2,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

f21 $1,101255 ; AND between two masks

mov $3,42
mov $2,$0
f31 $1,102131 ; Pick pixels from image and color. When the mask is 0 then pick from the image. When the mask is [1..255] then use the `default_color`.

mov $0,$1
