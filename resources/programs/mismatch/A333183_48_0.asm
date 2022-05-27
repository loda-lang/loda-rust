; A333183: Number of digits in concatenation of first n positive even integers.
; Submitted by Simon Strandgaard
; 1,2,3,4,6,8,10,12,14,16,18,20,22,24,26,28,30,32,34,36,38,40,42,44,46,48,50,52,54,56,58,60,62,64,66,68,70,72,74,76,78,80,82,84,86,88,90,92,94,97,100,103,106,109,112,115,118,121,124,127,130,133,136,139,142,145,148,151,154
; a(n) = Sum_{i=1..n} (1+floor(log_10(2*i))).

mov $1,$0
mov $0,2
add $1,1
mov $0,$1
trn $0,4
add $0,5
mod $2,2
add $0,$1
sub $0,5
