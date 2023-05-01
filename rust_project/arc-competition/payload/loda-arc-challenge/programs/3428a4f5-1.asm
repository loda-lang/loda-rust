; Submitted by Simon Strandgaard
; Program Type: simple

mov $5,$0
f11 $5,101001 ; get height
div $5,2

mov $4,$0
f21 $4,101221 ; get N bottom rows

mov $1,$5
f21 $0,101220 ; get N top rows

mov $1,$4
f21 $0,101254 ; xor
