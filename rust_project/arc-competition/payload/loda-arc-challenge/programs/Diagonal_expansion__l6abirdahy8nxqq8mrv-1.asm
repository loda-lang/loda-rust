; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $20,255
  mov $1,$0
  mov $2,$0
  f11 $2,101060 ; Image the 1 most popular colors, sorted by popularity
  f21 $1,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
  mov $10,$0
  mov $11,$1
  mov $12,$20
  f31 $10,102064 ; color of nearest neighbour pixel 'up left'
  mov $3,$10
  mov $4,$10
  mov $11,$1
  mov $13,$20
  f31 $10,102065 ; color of nearest neighbour pixel 'up right'
  mov $10,$0
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
  mov $15,$3
  mov $16,$20
  f31 $14,102101 ; Set pixel where the image has a pixel value different than the color parameter.
  mov $15,$4
  mov $16,$20
  f31 $14,102101 ; Set pixel where the image has a pixel value different than the color parameter.
  mov $15,$4
  mov $16,$20
  f31 $14,102101 ; Set pixel where the image has a pixel value different than the color parameter.
  mov $15,$6
  mov $16,$20
  f31 $14,102101 ; Set pixel where the image has a pixel value different than the color parameter.
  mov $0,$14
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "623ea044-1.asm"
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: DecrementSourceValueWhereTypeIsDirect
; mutate: SwapRows
