; ARC:5614dbcf
; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,3 ; number of noise colors to remove
f21 $0,101092 ; denoise type 3
f11 $0,101140 ; remove duplicates
