; ARC:f8ff0b80
; Submitted by Simon Strandgaard
; Program Type: simple

f11 $0,101230 ; convert image to histogram

mov $1,1 ; number of rows = 1
f21 $0,101221 ; take bottom row

mov $1,1 ; number of columns = 1
f21 $0,101226 ; remove left column

mov $1,1 ; 90 degrees cw
f21 $0,101170 ; rotate
