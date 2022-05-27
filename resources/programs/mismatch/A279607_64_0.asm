; A279607: Beatty sequence for e/2; i.e., a(n) = floor(n*e/2).
; 1,2,4,5,6,8,9,10,12,13,14,16,17,19,20,21,23,24,25,27,28,29,31,32,33,35,36,38,39,40,42,43,44,46,47,48,50,51,53,54,snip,82,84,85,MISMATCH
; 64 correct terms.

seq $0,329975 ; Beatty sequence for 1 + x + x^2, where x is the real solution of 1/x + 1/(1+x+x^2) = 1.
div $0,3
