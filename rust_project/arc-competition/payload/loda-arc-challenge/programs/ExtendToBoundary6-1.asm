; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $20,100
  mov $1,$0
  mov $2,$0
  f11 $2,101060 ; Image the 1 most popular colors, sorted by popularity
  f21 $1,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
  mov $10,$0
  mov $11,$1
  mov $12,$20
  f31 $10,102064 ; color of nearest neighbour pixel 'up left'
  mov $3,$10
  mov $10,$0
  mov $11,$1
  mov $13,$20
  f31 $10,102065 ; color of nearest neighbour pixel 'up right'
  mov $4,$11
  mov $10,$0
  mov $11,$1
  mov $13,$20
  f31 $10,102066 ; color of nearest neighbour pixel 'down left'
  mov $5,$10
  mov $10,$0
  mov $11,$1
  mov $13,$20
  f31 $10,102067 ; color of nearest neighbour pixel 'down right'
  mov $6,$10
  mov $14,$0
  mov $17,$20
  mov $16,$5
  mov $15,$4
  f41 $14,102100 ; Set pixel where two images agree on the pixel value.
  mov $17,$20
  mov $16,$6
  mov $15,$3
  f41 $14,102100 ; Set pixel where two images agree on the pixel value.
  mov $0,$14
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "1f876c06-1.asm"
; mutate: CallRecentProgram, no change
; mutate: ReplaceLineWithHistogram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: ReplaceSourceConstantWithHistogram
; mutate: ReplaceLineWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: IncrementSourceValueWhereTypeIsDirect
