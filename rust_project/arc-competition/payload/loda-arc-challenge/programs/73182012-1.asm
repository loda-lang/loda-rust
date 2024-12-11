; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  f11 $0,101160 ; Image: Trim border using histogram of border pixels
  mov $4,$0
  mov $5,$0
  f11 $4,101000 ; Get width of image
  f11 $5,101001 ; Get height of image
  div $4,2
  div $5,2
  mov $1,$4
  f21 $0,101220 ; get N top rows
  mov $1,$5
  f21 $0,101222 ; get N left columns
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "2013d3e2-1.asm"
; mutate: ReplaceLineWithHistogram
; mutate: ReplaceLineWithHistogram
