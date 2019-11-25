# Calculator

Simple mathematical expression evaluator (aka calculator) built using Nom, Pratt Parser, LLVM, Cranelift and Relm.

Every mathematical expression is parsed to lexical tokens using Nom. After initial parsing is complete, Pratt Parser algorithm is used to create AST (Abstract Syntax Tree) with right operator precedence.

There is simple interpreter implementation, which visits every AST node and computes result.

Alongside interpreter there is JIT compiler implementation with Cranelift and LLVM backends.

For end-user every mathematical expression is evaluated simultaneously by the interpreter and JIT compiler. JIT and Interpretator are racing to compute value first.

| Crate name        | Description                                                       |
| ----------------- | ----------------------------------------------------------------- |
| calculator_engine | Implementation of expression parser and execution modules         |
| calculator_cli    | Simple CLI interface that reads input from command line arguments ![CLI](https://i.imgur.com/2MztNbE.gif) |
| calculator_repl   | REPL with built-in basic syntax highlighting ![REPL](https://i.imgur.com/VPv3CuY.gif)                     |
| calculator_gtk    | Simple cross-platform GTK GUI ![GTK UI](https://i.imgur.com/2kOWsZY.gif)                                    |
