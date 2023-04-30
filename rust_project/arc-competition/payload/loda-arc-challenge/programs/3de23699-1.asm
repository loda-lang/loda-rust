; ARC:3de23699
; Submitted by Simon Strandgaard
; Program Type: simple

mov $10,$0
f11 $10,101060 ; most popular color

f11 $0,101160 ; trim

; get corner pixel color
mov $3,$0
mov $4,0
mov $5,0
f31 $3,101002 ; get top/left pixel
; $3 now holds the color of the corner pixel

; remove border
mov $1,1 ; number of columns = 1
f21 $0,101224 ; remove top row
f21 $0,101225 ; remove bottom row
f21 $0,101226 ; remove left column
f21 $0,101227 ; remove right column

; replace colors with the corner pixel color
mov $1,$10
mov $2,$3
f31 $0,101051 ; replace colors other than
