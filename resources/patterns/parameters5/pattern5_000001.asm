; PATTERN-NAME: ?

mov $1,2 ; source=parameter 0
pow $1,$0
mul $1,3 ; source=parameter 1
div $1,4 ; source=parameter 2
mul $1,2 ; source=parameter 3
add $1,6 ; source=parameter 4
mov $0,$1

; A005053: 5,30,100,2,1
; A013748: 1331,36,47880,14630,11
; A013749: 1331,36,47880,160930,121
; A013753: 2197,2,4392,371124,169
; A013756: 3375,196,661304,50610,15
; A013757: 3375,169,661304,759150,225
; A013761: 4913,196,962752,1419568,289
; A048474: 2,$0,2,3,1
; A048476: 2,$0,2,5,1
; A048478: 2,$0,2,7,1
; A048480: 2,$0,2,9,1
; A067403: 9,5,36,4,1
; A077842: 3,8,52,3,1
; A082365: 8,2,3,2,1
; A085287: 3,7,8,3,1
; A088556: 4,32,30,5,1
; A090860: 2,4,6,48,24
; A092164: 6,36,210,10,1
; A092165: 6,36,84,4,2
; A096019: 3,5,8,4,3
; A108983: 6,4,7,2,1
; A115342: 2,$0,2,64,1
; A116973: 3,6,16,9,1
; A146529: 2,3,4,2,6
