# How `parol` works

`parol` first transforms the input grammar into an expanded form where optional expressions, groups and repetitions are substituted by equivalent production sets. Then it analyzes this pre-transformed input grammar for several properties that prevent a successful processing. Those properties are

* Left-recursions
* Non-productive non-terminals
* Unreachable non-terminals

If there are no objections against the input grammar the next step is to left-factor the grammar that was produced by the previous expansion. This step is crucial for decreasing the number of necessary lookahead symbols.

This finally transformed grammar is the basis for the parser generation and can or better should be written to file for later reference. By convention this expanded grammar is stored to files names \<original-name\>-exp.par. Thus it is often useful to use this expanded grammar with any tool, because it is checked and left-factored. Also because this processed grammar is the basis for parser generation, you have to use it in this form in your grammar processing backend.

The actual parser generation then starts witch generating the lookahead automata for the non-terminals. In this phase it determines if the grammar is LL(k) for *k* starting with 1 and increasing it by one until a solution is found or the maximum lookahead size is exceeded. If your grammar is more than LL(5) the needed amount of processing power and memory consumption makes it inefficient to work with. In such a case you should rework your grammar design thoroughly. Or you can use a super fast machine to generate your parser's sources and compile and run the generated parser on an ordinary one. Internally the maximum lookahead size is currently limited to 10 though.

To determine if your grammar is LL(k) `parol` generates equation systems for both FIRST(k) and FOLLOW(k) sets and tries to solve them iteratively until a fix point is reached which indicates the solution. This is the most expensive task for `parol`.

If a solution is found `parol` generates all necessary data to feed the scanner and parser with. Based on this data `parol` then generates two source files.

The first one contains all scanner and parser data. The second one provides two traits. The first of these traits is important for the user's grammar processing. It contains for each production an empty default implementations of the corresponding semantic action. The semantic actions of the user can be provided by implementing this trait and providing own implementations for any production needed. The trait's name can be defined per command line argument.

The second trait in this file provides bindings of semantic actions so that the parser can call them via production number during parse time. It's name is always `UserActionsTrait`.
  