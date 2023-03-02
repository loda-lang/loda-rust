; Submitted by Simon Strandgaard
; Program Type: simple

mov $5,$0
f11 $5,102141 ; Compare with the pixels above,below,left,right and count how many have the same color as the center.

; We are only interested in pixels where there are 3 or more neighbour pixels that are the same.
mov $6,3 ; Ignore the pixels where count is 0, 1, 2
f21 $5,101253 ; Convert to a mask image by converting `pixel_color >= threshold_color` to 1 and converting anything else to to 0.

mov $7,$0 ; image
mov $8,$5 ; mask
f21 $7,101231 ; Histogram of image using a mask. Only where the mask is non-zero, are the image pixels added to the histogram.

; take the most popular color from the histogram
mov $8,0
mov $9,1
f31 $7,101002 ; get x=0, y=1
; $7 is now the repair_color

mov $10,$0 ; image that is to be repaired
mov $11,$7 ; repair color
f21 $10,102150 ; Fix damaged pixels and recreate simple repeating patterns.

mov $0,$10
