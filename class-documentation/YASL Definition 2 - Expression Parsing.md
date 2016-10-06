**CSC 426 - Compilers**

**YASL Definition for Project \#2**

**Expression Parser Rules**

This definition uses the same tokens from project \#1 (the YASL
definition handout includes ALL tokens, but you should only need to
recognize the ones from project \#1 for this project).

The grammar rules for the subset of YASL you need for this project are
as follows. The notation here encloses variables in &lt; &gt; brackets.
The terminals are tokens; the assumption is that for keywords and
operators, the token name matches the name of the keyword or operator.
If the expression is not clear, please ask.

&lt;Program&gt; -&gt; PROGRAM ID SEMICOLON &lt;Block&gt; PERIOD

&lt;Block&gt; -&gt; &lt;ConstDecls&gt; BEGIN &lt;Statements&gt; END

&lt;ConstDecls&gt; -&gt; &lt;ConstDecl&gt; &lt;ConstDecls&gt;

&lt;ConstDecls&gt; -&gt; ε

&lt;ConstDecl&gt; -&gt; CONST IDENTIFIER ASSIGN NUM SEMICOLON

&lt;Statements&gt; -&gt; &lt;Statement&gt; &lt;Statements&gt;

&lt;Statements&gt; -&gt; ε

&lt;Statement&gt; -&gt; PRINT &lt;Expression&gt; SEMICOLON

&lt;Expression&gt; -&gt; &lt;Expression&gt; PLUS &lt;Term&gt;

&lt;Expression&gt; -&gt; &lt;Expression&gt; MINUS &lt;Term&gt;

&lt;Expression&gt; -&gt; &lt;Term&gt;

&lt;Term&gt; -&gt; &lt;Term&gt; STAR &lt;Factor&gt;

&lt;Term&gt; -&gt; &lt;Term&gt; DIV &lt;Factor&gt;

&lt;Term&gt; -&gt; &lt;Term&gt; MOD &lt;Factor&gt;

&lt;Term&gt; -&gt; &lt;Factor&gt;

&lt;Factor&gt; -&gt; NUMBER

&lt;Factor&gt; -&gt; IDENTIFIER
