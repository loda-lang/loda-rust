; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,103
mov $84,104
mov $85,114
lps $80
  mov $0,$$81
  mov $1,$$85
  mov $3,$0
  mov $4,1
  f21 $3,101170 ; Image: Rotate by x * 90 degrees
  mov $5,$3
  mov $6,$2
  f21 $5,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.
  mov $7,$3
  mov $8,$5
  f21 $7,102230 ; Extract pixels where the mask value is non-zero.
  mov $9,$7
  mov $10,$$83
  mov $11,$$84
  mov $12,$1
  f41 $9,102241 ; Transfer pixels from one layout to another layout, ReverseOddRows.
  mov $0,$9
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
  add $84,100
  add $85,100
lpe

; template: "cdecee7f-1.asm"
; mutate: CallLeastPopularProgram, no change
; mutate: IncrementSourceValueWhereTypeIsDirect
