; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $20,$0
  f11 $20,101060 ; Image the 1 most popular colors, sorted by popularity
  mov $5,$0
  mov $6,3
  f21 $5,101220 ; get N top rows
  f21 $5,101222 ; get N left columns
  mov $6,$0
  mov $7,3
  f21 $6,101220 ; get N top rows
  f21 $6,101223 ; get N right columns
  mov $7,$0
  mov $8,3
  f21 $7,101221 ; get N bottom rows
  f21 $7,101222 ; get N left columns
  mov $8,$0
  mov $9,3
  f21 $8,101221 ; get N bottom rows
  mov $2,$20
  mov $0,$5
  mov $1,$6
  mov $2,$20
  f31 $0,101150 ; Image: Overlay another image by using a color as mask
  mov $1,$7
  f21 $8,101223 ; get N right columns
  f31 $0,101150 ; Image: Overlay another image by using a color as mask
  mov $1,$8
  mov $2,$20
  f31 $0,101150 ; Image: Overlay another image by using a color as mask
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "bc1d5164-1.asm"
; mutate: CallLeastPopularProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: CallMediumPopularProgram, no change
; mutate: SwapRows
