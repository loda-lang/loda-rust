; Submitted by Simon Strandgaard
; Program Type: simple

; $0 is the image that is to be repaired
mov $1,1 ; repair color
f21 $0,102151 ; Repair damaged pixels and recreate big repeating patterns such as mosaics.
