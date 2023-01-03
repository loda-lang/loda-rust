; ARC:a416b8f3
; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,$0
  f21 $0,101030 ; Image.hstack. horizontal stack of 2 images
  mov $$82,$0
  add $81,10
  add $82,10
lpe

; template: "6d0aefbc-1.asm"
; mutate: ReplaceTargetWithHistogram, no change
; mutate: ToggleEnabled
