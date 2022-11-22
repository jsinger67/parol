MODULE fibonacci;


IMPORT Out, Modules;


VAR
    n       : LONGINT;
    Arg0    : LONGINT;


PROCEDURE getFib (n : LONGINT) : LONGINT;
    VAR result : LONGINT;
BEGIN
    IF n = 0 THEN
        result := 0
    ELSIF n = 1 THEN
        result:= 1
    ELSE
        result := getFib(n-1) + getFib(n-2)
    END;
    RETURN result
END getFib;


BEGIN
    IF Modules.ArgCount # 2 THEN
        Out.String("one argument needed"); Out.Ln;
        HALT(1);
    END;
    Modules.GetIntArg(1, Arg0);
    Out.Int(getFib(Arg0), 0); Out.Ln;
END fibonacci.
