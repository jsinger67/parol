MODULE for;

IMPORT Out;

VAR
    i : INTEGER;


BEGIN
    Out.String("i is "); Out.Int(i, 0); Out.Ln;
    Out.String("For loop started"); Out.Ln;
    FOR i := 0 TO 10 DO
        Out.String("i : "); Out.Int(i, 0); Out.Ln;
    END;
    Out.String("For-By loop started"); Out.Ln;
    FOR i := 0 TO 10 BY 2 DO
        Out.String("i : "); Out.Int(i, 0); Out.Ln;
    END;
END for.
