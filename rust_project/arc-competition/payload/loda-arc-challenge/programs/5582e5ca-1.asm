; ARC:5582e5ca
; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,$0
  f11 $1,101060 ; Image the 1 most popular colors, sorted by popularity
  mov $2,$1
  f31 $0,101051 ; Image: replace colors other than x with color y
  mov $$82,$0
  add $81,10
  add $82,10
lpe

; template: "9565186b-1.asm"
; mutate: SetSourceToDirect
