Draft
Interview with Christian Krause, creator of LODA

# The big picture, what is LODA?



# What is OEIS?

OEIS hold around 350.000 integer sequences.
Examples: 
- The prime numbers.
- Fibonacci.
- The decimals in PI.


# In depth, what is LODA?

What if the OEIS database with 350.000 integer sequences was used as training data for an AI?
Would the AI be able to generate programs, just by looking at a few terms?

That is the vision.

You can help by mining new programs, or improving existing programs.

LODA runs on Raspberry Pi, Linux, Windows, macOS.
It cannot run on Android, iOS.


# How many LODA programs are there?

Around 61.000 programs. That's about 17% of the OEIS database.

However some of them are false positives. And there may be a mismatch after lots of terms.

The LODA programs can themselves be used as training data for other AI experiments.


# What discoveries have been made so far?


# Mining, how can people contribute to LODA?


# Community, join us?

Join us on slack.

Modify the miner.

Manually write programs, that the AI can learn from.


# What was the initial idea?

- When did you start coding on LODA?
- Did it have the name LODA from day 1?
- Did you make a proof of concept, and threw it out, and started over again?
- Did you do similar projects?


# What kind of programs did first iteration of LODA generate?

Small programs with a few instructions.

Later programs got to depend on other programs.


# How many program have been manually coded by a human?

Does the AI learn from a manually coded program? Isn't that like cheating?
Wouldn't it be better for the AI to discover the program on its own?


# What has the biggest challenges been so far?

Mutating programs in a way that minimizes the number of guesses.


# How does the LODA miner work?


# Mining: Why does it matter improving existing programs?

If the old program is terrible slow.

Then a new program gets mined that is much faster. Perhaps an entire loop has been eliminated.

These optimizations can lead to new discoveries in math.


