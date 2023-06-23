; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $3,$0
  f11 $3,101060 ; Image the 1 most popular colors, sorted by popularity
  mov $4,$0
  mov $5,4
  f21 $4,101092 ; Denoise type3. denoise noisy pixels. Takes a 2nd parameter: number of repair iterations.
  mov $6,$4
  mov $7,$3
  f21 $6,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.
  mov $8,$6
  mov $9,$3
  mov $10,$0
  f31 $8,102130 ; Pick pixels from color and image. When the mask is 0 then pick the `default_color`. When the mask is [1..255] then pick from the image.
  mov $0,$8
  f11 $0,101160 ; Image: Trim border using histogram of border pixels
  mov $$82,$0
  add $81,100
  add $82,100
  mov $0,$8
lpe

; template: "1f85a75f-1.asm"
; mutate: InsertLineWithHistogram, no change
; mutate: IncrementSourceValueWhereTypeIsConstant
; mutate: CopyLine
