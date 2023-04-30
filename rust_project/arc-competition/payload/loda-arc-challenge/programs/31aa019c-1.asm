; ARC:31aa019c
; Submitted by Simon Strandgaard
; Program Type: simple

mov $1,$0
f11 $1,101070 ; most unpopular color
mov $2,0 ; background color
f31 $0,101051 ; replace colors other than
mov $1,2 ; outline color
mov $2,0 ; background color
f31 $0,101080 ; draw outline
