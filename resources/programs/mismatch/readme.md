These are invalid programs that compute some of the initial terms, but after a while there is a mismatch.

A program that sort-of outputs an invalid sequence, is useful as training data.

Rejecting can work like this:
- Empty dictionary.
- Loop over all the program in the mismatch dir:
- For each program compute 40 terms and compute a hash, add the hash to the dictionary.
When mining and a potential candidate program satisfies the 40 initial terms, 
then also lookup the 40 terms among the mismatches and if so either keep it around or reject it.
