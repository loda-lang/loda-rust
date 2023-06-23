; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,$0
  f11 $1,101060 ; Image the 1 most popular colors, sorted by popularity
  mov $2,$0
  f11 $2,101000 ; Get width of image
  mov $3,6
  f20 $2,1033 ; Assert input[0] is less than or equal to input[1].
  mov $3,$0
  f11 $3,101001 ; Get height of image
  mov $4,5
  f20 $3,1033 ; Assert input[0] is less than or equal to input[1].
  mov $7,0
  mov $6,$3
  mov $5,$2
  f31 $5,101010 ; Create new image with size (x, y) and filled with color z
  mov $10,$0
  mov $11,$1
  f21 $10,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.
  mov $11,$0
  mov $12,$5
  f31 $10,102110 ; Create a big composition of tiles. When the mask is 0 then pick `tile0` as tile. When the mask is [1..255] then pick `tile1` as tile.
  mov $0,$10
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "c3e719e8-1.asm"
; mutate: InsertLineWithHistogram, no change
; mutate: IncrementSourceValueWhereTypeIsConstant
