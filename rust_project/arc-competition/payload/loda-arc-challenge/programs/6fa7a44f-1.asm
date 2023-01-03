; ARC:6fa7a44f
; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,$0
  f11 $1,101190 ; Image: flip x
  mov $1,$0
  f11 $1,101191 ; Image: flip y
  f21 $0,101040 ; Image.vstack. vertical stack of 2 images
  mov $$82,$0
  add $81,10
  add $82,10
lpe

; template: "3af2c5a8-1.asm"
; mutate: ReplaceLineWithHistogram, no change
; mutate: ReplaceLineWithHistogram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: ToggleEnabled
